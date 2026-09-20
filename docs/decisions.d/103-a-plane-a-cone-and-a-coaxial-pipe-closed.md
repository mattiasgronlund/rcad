## 103. A plane, a cone and a coaxial pipe, closed without running, recorded 2026-09-15

### 103.1 It earned nothing, and here is how that is known

§53 was planned on 2026-09-05 in ten subsections — a name, six corrections, a
falsifiable sentence, five lanes, two gates, a spike and a prediction. No lane
ran. It is closed here on the decision of 2026-09-15, and the closing report is
that the unit is entirely unstarted.

That is not inferred from silence. Five things say it, and each was checked:

1. **No section after §53 cites any of §53.1–§53.10.** `just decisions-find
   '§53\.[0-9]'` answers with §53's own subsections and nothing else.
2. **The two gates appear nowhere outside §53.** `decisions-find 'G1|G2'`
   reaches §53.4, §53.7 and one unrelated use in §96.5. A gate that ran would
   have a section; §45, §47 and §49 are what that looks like.
3. **`where_one_plane_and_two_curved_surfaces_meet` is still at
   `crates/rcad-geom/src/three_surfaces.rs:543`** — the exact line §53.3 cited
   ten days ago, as the place the coaxial cone-and-pipe road was to go.
4. **The word `coaxial` does not occur in `three_surfaces.rs`.** Lane A's
   reduction — two curved surfaces crossing in a conic put that conic in a
   plane, so `{P, S1, S2}` becomes `{P, Q, S1}` — is absent.
5. **`MeetOfThree::At` still documents its points as "running along the line the
   two planes cross in".** §53.3's second falsifier was that this order clause
   be replaced by one that survives two parallel planes. It was not.

So the sentence §53.3 set out to earn — *where three surfaces meet is worked out
when a cone and a pipe are proved coaxial and one plane cuts them* — is not
earned, and the refusal it was going to remove still stands.

### 103.2 What the nine days went to instead

§55 through §102, recorded 2026-09-06 to 2026-09-14: defect fixes found by the
fuse and the marcher (§78, §80, §82, §85, §87, §89, §93, §94), the nightly fuzz
apparatus and its failures (§81, §90, §91, §95, §96, §98, §101), and tooling,
gate and cost work (§77, §79, §83, §92, §99, §100, §102).

**None of it was wasted and none of it was §53.** That is the finding. The unit
was a plan for pushing the kernel's answers forward; the nine days were spent
making the kernel's existing answers trustworthy and the sessions that produce
them affordable. Both are work this project wants. What it did not have was
anything that noticed the plan had been abandoned — which is §102's whole
subject, and §53 sitting unclosed for nine days is the sharpest example of the
thing §0 now exists to prevent.

### 103.3 Two predictions this leaves unscored

**§52.13 is unscored, and stays unscored.** §53.7 charge 3 was to be how §53
scored it: *mutate what each load-bearing test claims to watch, and confirm it
fails*. Charge 3 never ran. §52.13's prediction — that a test would be blind,
its number coming from precisely the test that was named — therefore has no
verdict, and inventing one now from a gate nobody ran would be exactly the
offence §52 was recording. It is carried to §104.6.

**§53.10 is inherited, not scored.** It was always a bet about the *next* unit:
*the next unit's largest error will be in a claim its spike made that no merged
test carries.* The next unit is §104, so it goes there unchanged.

### 103.4 What was learned about running the work

The six units before this one each closed with a section. This one needed a
person to be asked before anyone could tell it had not. A plan is cheap to write
and there is no cost anywhere to leaving one open, so the only thing that closes
a unit is somebody looking — and until §0 there was nowhere for the looking to
be recorded when it found nothing.

The cheap countermeasure is already in place and is not a new rule: `just
status` prints §0 and its age. A unit that has been open a fortnight with no
section citing it is visible in one call to anybody who runs it.

