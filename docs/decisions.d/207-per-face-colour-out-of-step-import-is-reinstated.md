## 207. Per-face colour out of STEP import: the demand is reinstated, and what it actually costs — 2026-09-23

*rscene filed the demand on 2026-09-23 (session `rscene-18`), citing its own §7
as amended and its owner's answer Q2 in its §64. Its §7 had originally decided
**not** to file, on its §8.2 ground that a demand must name consumer code that
cannot be written without it — and rscene's fallback was written. Mattias
overrode that: rscene's requirement 2.1.4 is to be met rather than reported
around. Every number below was measured here, over the corpus at
`RCAD_STEP_CORPUS`, and none of it is quoted from rscene.*

### 207.1 What this reverses

`docs/consumers.md` carried a **withdrawal notice** for exactly this demand. It
was withdrawn, not refused, and the ground was §13: rscene had no consumer, so a
part with no named consumer is not built. That ground is gone — the consumer is
named and its owner has asked for it — so the notice is now false, and a later
builder reading it would conclude the matter was closed. It is reinstated in the
same commit as this section, and points here.

The withdrawal notice kept four design inputs "so that a later builder does not
have to re-derive them". They were correct as far as they went, and they were
the whole of what the demand ever specified. What follows is what they did not
say: what it costs.

### 207.2 The reader already parses this, and the doc comment is narrower than it reads

`import.rs`'s `shapes_this_reader_did_not_take` says of styling that "this
reader does not parse them", and rscene read that as the reader walking past
styling entities. That is true of *interpretation* and false of *holding*:
`StepFile` is a generic Part 21 graph, every entity in the file is parsed and
addressable whatever its keyword, and the map in question is built from
`file.how_many_of_each_type()` — a census of the whole file.

So carrying styling adds **no tokenizer work and no new schema kinds**. The
chain is a walk of references over helpers `Reading` already has
(`reference_at`, `references_at`, `real_at`, `keyword_of`), confirmed end to end
in `LED_D2.0mm_W4.8mm_H2.5mm_FlatTop.step`:

```
STYLED_ITEM('color',(#502),#17)
  -> PRESENTATION_STYLE_ASSIGNMENT((#503))
  -> SURFACE_STYLE_USAGE(.BOTH.,#504)
  -> SURFACE_SIDE_STYLE('',(#505))
  -> SURFACE_STYLE_FILL_AREA(#506)
  -> FILL_AREA_STYLE('',(#507))
  -> FILL_AREA_STYLE_COLOUR('',#508)
  -> COLOUR_RGB('',0.699999988079,0.10000000149,5.000000074506E-002)
```

### 207.3 The identity exists today, and `build` drops it on one line

rscene's message named identity as the hard part — a colour is useless without
a `FaceKey` to hang it on, and rscene deliberately cannot reach the entity graph
to re-derive one. It is not the hard part, because rcad already keeps the
correspondence for its own refusal reporting:

- `Walk::describe_the_face` pushes the `ADVANCED_FACE`'s `EntityId` onto
  `face_names` immediately after `SolidBuilder::add_face` succeeds — same order,
  one entry per face taken. A face the probe walk skips is never pushed, so the
  index stays aligned.
- `SolidBuilder::write_into` produces `Written { solid, edges, faces }`, whose
  doc comment says both lists are "kept in the order they were described" — the
  property `which_described_face_the_sweep_meant` already relies on.

Same index, both directions. `face_names[i]` is the file's face and
`Written.faces[i]` is the slot it became.

`SolidBuilder::build` then throws that away:

```rust
let solid = self.write_into(model, shell).solid;
```

The second write's `.faces` is discarded at the point of use. **The one
structural change this demand needs is in `rcad-topo`'s public API, not in the
STEP reader** — `build` handing back the faces it wrote as `FaceKey`s — which is
the opposite of where the cost was expected to sit. Recorded because a builder
sizing this from the STEP side will look in the wrong crate.

### 207.4 Scope is the real cost, and it is a publication question

Measured across the corpus, resolving every `STYLED_ITEM`'s third attribute to
the keyword it names.

**KiCad, 323 files, every one of them styled.** 48,426 styled items:

| target | count |
|---|---|
| `ADVANCED_FACE` | 48,374 |
| `MANIFOLD_SOLID_BREP` | 51 |
| `BREP_WITH_VOIDS` | 1 |

For rscene's own corpus that is a clean per-face map. The `BREP_WITH_VOIDS` is
moot — rcad refuses that file by name before any colour question arises.

**NIST's AP242 files are not that shape.** `nist_ctc_01_asme1_ap242-e1.stp`
styles 214 `LINE`, 117 `ADVANCED_FACE`, 90 `CIRCLE`, 57 `CYLINDRICAL_SURFACE`,
56 `PLANE`, 14 `B_SPLINE_CURVE_WITH_KNOTS`, 4 `CONICAL_SURFACE`, 1
`MANIFOLD_SOLID_BREP`, 1 `CARTESIAN_POINT`. Styling a surface entity is
one-to-many onto the faces sitting on it; styling a curve is an edge, which
rscene's sink cannot hold at all.

Also present and not hypothetical: `OVER_RIDING_STYLED_ITEM` in 32 KiCad and 4
NIST files, `DRAUGHTING_PRE_DEFINED_COLOUR` — named colours rather than RGB — in
25 NIST files, `SURFACE_STYLE_TRANSPARENT` and `SURFACE_STYLE_RENDERING` in 1
KiCad and 5 NIST.

So a per-face map answers KiCad completely and AP242 partially. **What is
published has to say which**, for the same reason
`shapes_this_reader_did_not_take` carries its own warning: an absent entry must
not read as "this file had no colour when it had colour rcad could not place".
Building a second instance of the trap §56 fixed would be the defect here, not
the partial coverage.

### 207.5 The colour space is not declared, and rcad will not guess it

rscene asked to be told which colour space the value is in rather than guess.
The honest answer is that the file does not say: ISO 10303-46's `colour_rgb` is
three reals and names no transfer function. The numbers above are float32
`(0.7, 0.1, 0.05)` — what a colour picker produces — and the convention among
exporters is that they are sRGB-encoded, but that is a convention and not
something rcad can check per file.

§42.9 governs: nothing is healed. rcad hands back the file's own three numbers
unchanged and says in the doc comment that no colour space is declared and what
the exporter convention is. The decode stays rscene's, which its §5 already
claims. Asserting sRGB as a fact rcad had not measured would be §16.10's error
in a different currency — a sentence standing where a demonstration belongs.

### 207.6 Accepted, and not built today

§13 no longer closes this: the consumer is named. But rcad's §0 unit is the
near-parallel crossing, a new unit is Mattias's call, and rscene said there is no
rush and no pin bump implied. So what lands now is the record and nothing else,
and rscene's `NoFaceColourCameWithTheseSolids` report stays correct until the
work does.

### 207.7 The shape, settled with rscene the same day

Two questions went back to rscene and both came back measured against its sink
rather than answered from taste. The shape below is fixed; only the schedule is
open.

**The 51 whole-solid styles surface separately, never expanded onto faces.**
rscene's sink is `Appearance { colour: Colour, per_face: Vec<(FaceKey, Colour)> }`
— one instance colour and a list of per-face overrides — so the two cases
already have two homes. Expanding would put N entries in `per_face`, and
`Appearance::overrides()` is a public accessor an app reads: it would then
report N overrides the file never wrote. That is this side's healing objection
arriving as a *wrong answer* on the other side of the seam, which is a better
argument than the one made here. rscene adds a second: the instance colour is
the app's to set and may already be set, so rscene cannot merge a file's solid
colour into it on the app's behalf, and the two must not arrive pre-merged.

**Faces only, and the rest is counted.** rscene checked rather than assumed:
`edges.rs` holds no colour at all, and in `lines.wgsl` an edge takes its colour
from the uniform, one per draw, with only an overlay segment carrying its own.
A per-edge colour would need a new edge attribute, a second path through the
lines pipeline and a new `Appearance` field — a part with no consumer, refused
by rscene's own rule as §13 would refuse it here.

**The count is load-bearing, and is the half rscene values most.** Faces alone
would leave 207.4's trap open. What closes it is publishing, beside the map, how
many styled items were *not* faces and what they were — so the seam can say
"this import carried C face colours and walked past S styled items that were not
faces, of which E were curves". rscene's §7 forbids it from claiming a file
*had* colour that was lost, because it has no instrument for that; this count is
that instrument. So the count is not a diagnostic extra — it is what lets the
downstream sentence be true, and it ships with the map or the map is not worth
shipping.

**Colour space: three reals and the doc comment, no typed marker.** rscene's
answer is that a marker would change nothing it does — §5 already makes the
decode its own, so it sRGB-decodes on arrival whichever shape arrives, and a
type saying "no transfer function was declared" is a type it reads and ignores.
So 207.5 stands as written, and rscene will **cite** the doc-comment sentence
rather than restate it, which `docs/consumers.md` allows and which makes that
sentence a thing rcad must retire if it stops being true. If rcad ever meets a
file that does declare a transfer function, that is when a marker earns its
place; adding it then is smaller than carrying it unused until then.

**The identity needs nothing from rscene.** `FaceKey` is rcad's own type and
rscene names rcad only in `kernel.rs`; everything downstream speaks the bare
handle. A map keyed by `FaceKey` needs no translation anywhere in rscene.

### 207.8 The census is cut by what the consumer can do, not by entity kind

207.7 settled that the count ships with the map. rscene then specified its
shape, and the specification is better than the one this side would have
written: **cut the count by what rscene can do about each case**, because that
is the only axis its report can act on. A curve colour is about rscene's own
limits and always will be. A colour on a `CYLINDRICAL_SURFACE` **is** per-face
colour in substance, addressed one-to-many — a user looking at a flat grey
cylinder the file coloured is looking at a confidently wrong picture, which is
the thing §1 ranks below a refusal. Folded together, that sentence cannot be
said. So the rows do not fold.

rscene also named a case neither side had counted: a styled item whose target
**is** an `ADVANCED_FACE` — identity in hand, join works — whose chain reaches
`DRAUGHTING_PRE_DEFINED_COLOUR` rather than three reals. The colour is missing,
not the face. That is the sharpest sentence available to the seam and a
different failure from every other row.

Measured here over all 367 corpus files, resolving each `STYLED_ITEM`'s target
and walking its style chain to whatever colour entity it reaches:

| row | what rcad can do about it | count | share |
|---|---|---|---|
| 1 | carried — a face, and three reals | 50,866 | 84.40% |
| 2 | a face named, with no reals (`DRAUGHTING_PRE_DEFINED_COLOUR`) | 1,332 | 2.21% |
| 3 | a surface several faces sit on | 1,390 | 2.31% |
| 4 | a curve or an edge | 5,712 | 9.48% |
| 5 | anything else, by its own keyword | 970 | 1.61% |

Row 5 is named rather than bucketed, which is rscene's §42 rule against a
wildcard arm swallowing a new variant, applied to a census: `CARTESIAN_POINT`
453, `COMPLEX_TRIANGULATED_FACE` 272, `MANIFOLD_SOLID_BREP` 87,
`AXIS2_PLACEMENT_3D` 65, `GEOMETRIC_CURVE_SET` 49, `SHELL_BASED_SURFACE_MODEL`
27, `MAPPED_ITEM` 4, `BREP_WITH_VOIDS` 1, and **12 that resolve to nothing at
all** — a styled item pointing at an absent `#id`, which is
`Refusal::AReferencePointsNowhere`'s shape arriving somewhere it currently
cannot be raised. Four more reach no colour entity down any path. Both are
small and neither should be silent.

The 87 `MANIFOLD_SOLID_BREP`s are the whole-solid case 207.7 sends to
`Appearance::colour`; they sit in row 5 here only because this cut is by target
kind and that case already has its own home.

**Two cautions on these numbers, so a builder does not over-read them.** The
partition of keywords into "surface" and "curve" is this measurement's own, not
rcad's — row 3 and row 4 move if that set is drawn differently, and the built
census must pin its partition with a test rather than inherit this one. And
`OVER_RIDING_STYLED_ITEM` is a styled-item kind rather than a target kind: it is
orthogonal to this cut, and whatever it resolves to belongs in whichever row its
target lands in rather than becoming a sixth.

**What rcad still owns.** Whether a row-3 surface colour *could* be placed on
the faces sitting on it is rcad's judgement and a one-to-many mapping rcad owns.
The census only names the category; it does not resolve it.

### 207.9 One sentence this will retire

rscene quotes `import.rs:195`'s doc comment **verbatim**, by file and line, in
its §7 — as the evidence that this reader says out loud that it does not read
colour. That sentence moves when this lands, and a citation that stops
describing anything is worse than a paraphrase because its provenance is what
makes a reader trust it (`docs/consumers.md`). So landing this carries the
retirement obligation that file already sets out: rcad says the quote has
stopped describing anything, at the same time as it says the map exists, to
whichever rscene session is live. rscene then bumps its pin and walks its §10
list. That is the existing demand process and nothing here changes it.
