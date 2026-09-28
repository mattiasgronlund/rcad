## 347. A node where a plane through a cone's apex crosses its two rulings — 2026-09-28

*Row Q207 (`lane-q207-a-node-at-the-apex`, off `f254b984`). §283 found that a
block whose face `x = 0` runs through a pointed cone's apex is refused in all
six fuses, because `place_the_meets` puts no node where the plane's two
rulings cross, and named a fix in two parts (§283.5). This section is that fix
as it was built. Work in progress: the parts below that say so were not
finished when this was written.*

  347.1   Part 1 alone: the node, and what it swaps the refusal for

### 347.1 Part 1 alone: the node, and what it swaps the refusal for

`Arrangement::place_the_meets` now hands a pair of crossings with one partner
to `place_where_two_rulings_cross`, which puts a node at `Cone::apex` when the
pair is a plane and a cone that `a_plane_cuts_a_cone_in_rulings` says cut in
rulings. The apex is stored, so the node is exact, and both rulings' pickers
hold it: they are opposite half-spaces whose flat surface passes exactly
through the apex, and `rcad-geom` proves that before it names the pair.

§283.5 said part 1 alone might only swap the refusal for
`TwoCurvesTouchWithoutCrossing`. **It does.** With part 1 and nothing else,
the union with the cone first — the first fuse each test asks, and the only
one it reached before failing — refuses `TwoCurvesTouchWithoutCrossing` on
face `2v1`, **the cone's skin**, not on the block's face `8v1`, both for the
apex on the face and for the raised-floor control. The skin now has a
node at the apex with the two rulings through it, and the skin's own gradient
is nothing there, so no turn between them can be read.
