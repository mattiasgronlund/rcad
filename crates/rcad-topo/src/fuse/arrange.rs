//! One face against every crossing that cuts it, all at once.
//!
//! Never one crossing and then the next: the pieces would depend on the order
//! the crossings arrived in, which depends on the order of the arguments, and
//! §14.8's commutativity and associativity oracles would fail by construction
//! (§17.4). The cut surfaces are sorted into a canonical order on the way in, so
//! what comes out is a function of the face and the *set* of surfaces cutting
//! it.
//!
//! # Everything here is a pair of surfaces
//!
//! A face `F` sits on a surface `S`. Every curve that matters to it is
//! `S ∩ Q`: `Q` is the surface of the face across a boundary edge, or the
//! surface of a face cutting it. §17.4 said "pair of planes" and §31.3 changes
//! the one word. So:
//!
//! - every node of the arrangement is [`meet_of_three_surfaces`], which for
//!   three planes is bit-for-bit the Cramer solve that used to be here (§34.9);
//! - the anticlockwise order of two curves round a node is one exact
//!   determinant of the three surfaces' **gradients** there — §17.4's
//!   determinant of three normals, with `∇S` in each normal's place, which is
//!   the same derivation with flatness never used (§31.2 row 9);
//! - the order of points along a crossing is a comparison on the direction's
//!   dominant axis where the crossing is straight,
//!   [`which_comes_first_round_a_crossing`] where it closes, and
//!   [`along_a_bend`] where it runs on for ever and bends anyway.
//!
//! # How the pieces are found, and what a closed curve changes
//!
//! The arcs of the arrangement are drawn on the surface; each one is walked from
//! both sides, and the walks close into cycles with the region they bound on the
//! left. How many times the face's boundary winds round each cycle's region is
//! then a count that propagates across the arcs: crossing a boundary arc from
//! its right to its left adds one, crossing a cut arc adds nothing. The cells
//! wound once are the pieces.
//!
//! **That much is unchanged and none of it was ever about straight lines.** What
//! *was* about straight lines is how the walks are joined up, and it took three
//! forms, every one of them under a name that said nothing about it:
//!
//! - `drop_the_lumps_that_are_only_cuts` argued that "a cut line that runs
//!   through the face crosses its boundary to get there". True of a line and
//!   **false of a circle**: the circle a bar's wall cuts in a block's face lies
//!   wholly inside it and touches no boundary. Gone.
//! - `piece_around` followed one of a lump's own lines until it ran into an arc
//!   of the rest of the drawing, comparing positions along one axis at every
//!   turn. A closed curve has no axis to climb and the ray has nowhere to start.
//!   Gone.
//! - `pieces` read the first wire of the face as the outer one and everything
//!   else as a hole inside it. A ball's skin with a pipe through it has two
//!   boundary curves and neither is outer (§31.3). Gone.
//!
//! What replaces all three is one rule and it needs no ray and no construction:
//! **two walks in different lumps of the drawing bound the same cell exactly
//! when they lie on the same side of every crossing that is drawn in full.** A
//! crossing is drawn in full when none of its arcs was thrown away, which is
//! every cut crossing and every closed boundary edge; the side a walk lies on is
//! read off the half-edge where the walk uses that crossing, and off any of its
//! own nodes where it does not. Both are signs the kernel already had.

use core::cmp::Ordering;

use rcad_geom::{
    Cone, Cylinder, EdgeRun, MeetOfThree, NamedRun, Orientation, POINT_ALONG_A_CROSSING_ROUNDOFF,
    Picker, Plane, PlaneAgreement, Point, Sphere, Surface, SurfaceMeet, Vector,
    how_a_plane_leans_against_a_cone, intersect, meet_of_three_surfaces,
    where_a_closed_crossing_starts, where_a_cones_apex_stands_against_a_plane,
    which_comes_first_round_a_crossing,
};
use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use rcad_predicates::{
    Decision, Interval, Measures, ORIENT2D, ORIENT3D, Point as PredicatePoint, Policy,
    ProvenanceLog, orient2d, orient3d, orient3d_of_points,
};

use smallvec::SmallVec;

use crate::validate::{name_of_surface, side_of_surface};
use crate::{Error, FaceKey, Invalid, Result, VertexKey};

use super::bake::BakedFace;
use super::select::InPlane;
use super::{
    FaceMeet, MeetPoint, Sameness, as_exact_as_it_is, gave_up, must_decide, the_other_way,
};

/// Cut one face into the pieces its cut surfaces leave behind.
///
/// A face with nothing cutting it comes back as one piece: itself.
///
/// # Errors
///
/// [`Error::Undecidable`](crate::Error::Undecidable) when a decision the split
/// depends on could not be settled under `policy`,
/// [`Error::ACrossingThisCannotBeOrderedAlong`] when a curve the face is cut by
/// is one nothing here can put points in order along, and
/// [`Error::Invalid`](crate::Error::Invalid) when the face turns out to be
/// broken — two of its corners in the same place, say.
///
/// # Panics
///
/// If the arrangement comes out contradicting itself: a cell that is both inside
/// and outside the face, or a piece with no sides at all. Those are broken
/// internal invariants, and a corrupt solid is worse than a crash (§4.1).
pub(crate) fn split_a_face(
    face: &BakedFace,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Piece>> {
    let cuts = canonical_cut_surfaces(face);
    let arrangement = Arrangement::of(face, &cuts, policy, log)?;
    arrangement.pieces()
}

/// The stretches of the crossing `face.surface ∩ partner` that run through the
/// inside of `face`.
///
/// Used to tell a solid that crosses itself from one that merely has two faces
/// whose surfaces cross somewhere out in space (§17.4).
///
/// # Errors
///
/// The same as [`split_a_face`].
pub(crate) fn stretches_of_a_line_inside(
    face: &BakedFace,
    partner: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Stretches> {
    let arrangement = Arrangement::of(face, core::slice::from_ref(partner), policy, log)?;
    Ok(arrangement.stretches_inside())
}

/// One piece a face was cut into.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Piece {
    /// The input faces this came out of. Every piece has at least one ancestor,
    /// and nothing here is allowed to invent geometry that has none; more than
    /// one when coplanar faces of the input were joined into a single face on
    /// the way in. They are all faces of one solid.
    pub(crate) ancestors: Vec<FaceKey>,
    /// The surface, inherited from the ancestor bit for bit (§17.1, §31.3).
    pub(crate) surface: Surface,
    /// Which way round the ancestor's solid used it.
    pub(crate) orientation: Orientation,
    /// The boundary, every wire keeping the piece's material on its left.
    ///
    /// On a **flat** piece the first wire is the outer one and the rest are
    /// holes, which is the winding [`Model::validate`] asks for there. On a
    /// closed surface neither is outer and the order is the order the walks were
    /// found in — §31.3 settles that "outline and holes" is a thing a face
    /// stores rather than a fact about the geometry.
    ///
    /// [`Model::validate`]: crate::Model::validate
    pub(crate) wires: Vec<Vec<FaceMeet>>,
    /// Whether this piece is the **whole** of its surface.
    ///
    /// Then the one wire above is a *seam* and not a trimming: it is where the
    /// surface's own two coordinates were cut open, so every point of the
    /// surface is in the piece and no side of it says otherwise (§38.1).
    pub(crate) is_the_whole_surface: bool,
}

impl Piece {
    /// The plane this piece sits on.
    ///
    /// # Panics
    ///
    /// If the piece is curved. Only [`Stretches`] asks, and only about a
    /// straight crossing, which is a pair of planes.
    pub(crate) fn plane(&self) -> Plane {
        *self
            .surface
            .as_plane()
            .expect("only a flat piece is asked which plane it is on")
    }
}

/// Where a straight crossing runs through the inside of a face, as ranges of one
/// coordinate.
#[derive(Clone, Debug)]
pub(crate) struct Stretches {
    /// The coordinate axis the ranges are measured on: the one the crossing
    /// climbs fastest along.
    axis: usize,
    /// One range per stretch, each a pair of enclosures of its two ends.
    spans: Vec<[Interval; 2]>,
}

impl Stretches {
    /// Whether the two sets of stretches *provably* share more than a point.
    ///
    /// Proof only. Two stretches whose enclosures merely touch have not been
    /// shown to overlap, and reading that as an overlap would refuse solids that
    /// are perfectly sound — a pair of faces meeting at one corner, for
    /// instance.
    ///
    /// # Panics
    ///
    /// If the two were measured on different axes, which would make the numbers
    /// incomparable. Both come from the same pair of surfaces, so both pick the
    /// same axis, and an empty set of stretches has no axis to disagree about.
    pub(crate) fn provably_overlap(&self, other: &Stretches) -> bool {
        if self.spans.is_empty() || other.spans.is_empty() {
            return false;
        }
        assert!(
            self.axis == other.axis,
            "two stretches of the same crossing must be measured on the same axis"
        );
        self.spans.iter().any(|mine| {
            other.spans.iter().any(|theirs| {
                let (mine, theirs) = (certain_part(*mine), certain_part(*theirs));
                mine.0 < theirs.1 && theirs.0 < mine.1
            })
        })
    }
}

/// The stretch of the axis a span certainly covers, whatever its ends do inside
/// their boxes.
fn certain_part([from, to]: [Interval; 2]) -> (f64, f64) {
    let (low, high) = if from.hi() <= to.lo() {
        (from.hi(), to.lo())
    } else {
        (to.hi(), from.lo())
    };
    (low, high)
}

/// The cut surfaces of a face, in an order that depends on the set and nothing
/// else.
///
/// Sorted on the numbers each one stores so that the same set always arrives the
/// same way round, and the arrangement is therefore a function of the face and
/// the set (§17.4). Surfaces that are equal as stored numbers are dropped; two
/// *different* surfaces that happen to cut `S` in the same curve are found later
/// by [`Arrangement::the_same_crossing`].
fn canonical_cut_surfaces(face: &BakedFace) -> Vec<Surface> {
    let mut surfaces: Vec<Surface> = face.cuts.clone();
    surfaces.sort_by_key(name_of_surface_bits);
    surfaces.dedup();
    surfaces
}

/// The numbers a surface stores, as bits, with its kind first.
///
/// A total order over surfaces that depends on nothing but what each one holds,
/// and a key a corner can be named by. Sound for exactly the reason §32.5's trap
/// gives — two faces that are the same plane hold the same four numbers — and
/// **not** sound for a [`HalfSpace`](rcad_geom::HalfSpace), which §33.8 says has
/// no canonical form at all.
///
/// **It orders surfaces the same way `rcad-geom` does**, which is what stops
/// this being a second answer to a settled question (§23.8): the meet of three
/// surfaces puts its three in a fixed order of its own and counts `which` of
/// their meeting points this is along it, and a `which` counted one way beside a
/// triple sorted the other is two corners named as one. That is why the bits are
/// flipped rather than taken raw — raw bits sort every negative number after
/// every positive one, and `f64::total_cmp` does not.
pub(crate) fn name_of_surface_bits(surface: &Surface) -> [u64; 9] {
    let mut bits = [0u64; 9];
    let mut fill = |kind: u64, numbers: &[f64]| {
        bits[0] = kind;
        for (slot, number) in numbers.iter().enumerate() {
            bits[slot + 1] = in_the_order_a_float_compares(*number);
        }
    };
    match surface {
        Surface::Plane(flat) => fill(0, &flat.coefficients()),
        Surface::Sphere(ball) => {
            let centre = ball.centre().coordinates();
            fill(1, &[centre[0], centre[1], centre[2], ball.radius()]);
        }
        Surface::Cylinder(pipe) => {
            let origin = pipe.axis().origin().coordinates();
            let along = pipe.axis().direction().components();
            fill(
                2,
                &[
                    origin[0],
                    origin[1],
                    origin[2],
                    along[0],
                    along[1],
                    along[2],
                    pipe.radius(),
                ],
            );
        }
        Surface::Cone(cone) => {
            let apex = cone.apex().coordinates();
            let along = cone.axis().direction().components();
            fill(
                3,
                &[
                    apex[0],
                    apex[1],
                    apex[2],
                    along[0],
                    along[1],
                    along[2],
                    cone.slope(),
                ],
            );
        }
        // Eight numbers, one more than any other kind, and the row is nine long
        // for it: two tori about one axis with the same ring radius and
        // different tubes are two surfaces, and a key that left the tube out
        // would call them one.
        Surface::Torus(ring) => {
            let centre = ring.centre().coordinates();
            let along = ring.axis().direction().components();
            fill(
                4,
                &[
                    centre[0],
                    centre[1],
                    centre[2],
                    along[0],
                    along[1],
                    along[2],
                    ring.ring_radius(),
                    ring.tube_radius(),
                ],
            );
        }
        other => panic!("a surface nothing here can name: {other:?}"),
    }
    bits
}

/// The numbers a surface stores, with a plane's turned round where that makes
/// them smaller: a key for the surface's **geometry** and not for which way it
/// faces.
///
/// §31.3 splits "is it one surface", which is geometry, from "does it face the
/// same way", which is topology. This is the first half on its own, and the
/// commit is where it is needed: the two faces along one edge of the answer may
/// come from two solids pressed against each other, and then one names the
/// shared plane facing up and the other names it facing down. Keyed on the
/// stored numbers those are two edges where the model has one, and an edge with
/// one face along it is not a solid.
///
/// A plane reversed is the four coefficients negated exactly
/// ([`Plane::reversed`](rcad_geom::Plane::reversed)), so this rounds nothing —
/// and negating turns `0.0` into `-0.0`, which is a *different* `f64` and would
/// leave the two spellings of one plane still telling apart. Every zero is
/// turned back the right way up, and adding zero is what does it. A ball and a
/// pipe have no reversed form at all and are their own key.
pub(crate) fn name_of_the_surface_either_way_round(surface: &Surface) -> [u64; 9] {
    let Surface::Plane(flat) = surface else {
        return name_of_surface_bits(surface);
    };
    let written = flat.coefficients();
    let backwards = written
        .iter()
        .find(|coefficient| **coefficient != 0.0)
        .is_some_and(|coefficient| *coefficient < 0.0);
    let facing = written.map(|coefficient| {
        let turned = if backwards { -coefficient } else { coefficient };
        turned + 0.0
    });
    let mut bits = [0u64; 9];
    for (slot, number) in facing.iter().enumerate() {
        bits[slot + 1] = in_the_order_a_float_compares(*number);
    }
    bits
}

/// Three planes turned to face the one way, so that where they meet is a fact
/// about the three of them and not about which faces carry them.
///
/// §48, W5. Two solids flush against each other work the plane they share out
/// separately and one of them holds it reversed, so a corner on that plane is
/// asked for as `P, Q, R` by a face of one and as `−P, Q, R` by a face of the
/// other. [`meet_of_three_surfaces`] sorts its three on the numbers they store,
/// a reversed plane sorts to a different row, and a solve with its rows
/// permuted rounds a bit or two elsewhere. Which face happened to ask then
/// decided where the corner went, and a `fuse(A, B)` and a `fuse(B, A)` that
/// keep different copies of the shared face — which is the whole point of
/// keeping one — got corners a bit apart. §17.4 forbids that, and
/// `fuses_that_must_agree.rs` compares the two bit for bit with no tolerance.
///
/// The turning is exact: [`Plane::reversed`] negates all four coefficients and
/// negating twice comes back to where it started, so this picks one of the two
/// spellings and never a third thing. **The choice is which one, not what**: it
/// is the same plane either way, and every face on it goes on carrying its own
/// spelling for its own normal.
///
/// A triple with anything curved in it is handed over untouched. There, `which`
/// numbers the meeting points along the line the two planes cross in and turning
/// a plane round turns that line round, so the same point would be `which: 0`
/// under one spelling and `which: 1` under the other —
/// `assemble::names_of_the_three_surfaces` is the other half of this and says
/// the same.
fn three_planes_faced_the_one_way(surfaces: [Surface; 3]) -> [Surface; 3] {
    if !surfaces.iter().all(|surface| surface.as_plane().is_some()) {
        return surfaces;
    }
    surfaces.map(|surface| {
        let Surface::Plane(flat) = surface else {
            unreachable!("every one of the three was just checked to be a plane")
        };
        let backwards = flat
            .coefficients()
            .iter()
            .find(|coefficient| **coefficient != 0.0)
            .is_some_and(|coefficient| *coefficient < 0.0);
        Surface::Plane(if backwards { flat.reversed() } else { flat })
    })
}

/// The bits of a float, rearranged so that comparing them as whole numbers is
/// comparing the floats.
///
/// The sign bit comes first and every negative number is turned inside out, so
/// `-1.0 < 0.0 < 1.0` as `u64` exactly as it is with
/// [`f64::total_cmp`](f64::total_cmp).
fn in_the_order_a_float_compares(number: f64) -> u64 {
    let bits = number.to_bits();
    bits ^ ((((bits as i64) >> 63) as u64) | (1 << 63))
}

/// A face, the crossings drawn on it, and everything that follows from them.
struct Arrangement<'f> {
    face: &'f BakedFace,
    surface: Surface,
    crossings: Vec<Crossing>,
    nodes: Vec<Node>,
    arcs: Vec<Arc>,
    /// Whether the *whole* of each crossing is drawn: it closes, and none of its
    /// arcs was thrown away. Only such a crossing can say which side of itself a
    /// walk elsewhere on the surface is on — a straight one runs on past its
    /// outermost node, and two walks either side of that stretch are one cell
    /// with two different signs.
    drawn_in_full: Vec<bool>,
    /// Which crossing each cut surface handed in turned into, in the order they
    /// were handed in. `None` when the surface misses the face's own and cuts
    /// nothing.
    cut_crossings: Vec<Option<usize>>,
    /// Which crossing each boundary edge lies on, by wire and position.
    crossing_of_edge: Vec<Vec<usize>>,
    /// Which node each edge of each wire starts at, by wire and position.
    node_of_corner: Vec<Vec<usize>>,
    /// Which node each of them stops at.
    ///
    /// Read off the edge and never off the next one along: a wire with a seam
    /// left out of it is no longer a chain, and a closed edge stops where it
    /// started.
    node_of_the_end: Vec<Vec<usize>>,
    /// The half-edges leaving each node, sorted anticlockwise.
    leaving: Vec<Vec<usize>>,
    /// Which cycle each half-edge is walked by.
    cycle_of: Vec<usize>,
    /// The cycles, each a chain of half-edges with its region on the left.
    cycles: Vec<Vec<usize>>,
    /// How many times the face's boundary winds round each walk, counted inside
    /// its own lump alone. What a lump has no boundary arc in it cannot say, and
    /// that is what joining the lumps up settles.
    winding_of_cycle: Vec<Option<i32>>,
    /// Which cell of the surface each cycle bounds.
    cell_of: Vec<usize>,
    /// The cells, each one or more cycles.
    cells: Vec<Vec<usize>>,
    /// How many times the face's boundary winds round the region each cell is.
    winding: Vec<Option<i32>>,
}

/// One curve on the face: one of the curves where the face's surface crosses
/// another.
struct Crossing {
    partner: Surface,
    /// Which curve of the pair this is, as [`intersect`] named it. Carried
    /// rather than taken apart, because [`where_a_closed_crossing_starts`] wants
    /// exactly this and a second spelling of it is §23.8's shape again.
    run: NamedRun,
    /// How points on it are put in order.
    order: HowItRuns,
    /// Whether a face of the other group put this crossing here.
    cuts: bool,
    /// Whether any of it is inside the box round the face. A curve wholly
    /// outside cuts nothing off it and is left undrawn.
    reaches_the_face: bool,
}

impl Crossing {
    /// What picks this curve out of the others the pair names. Empty when the
    /// pair names one curve and there is nothing to pick.
    fn picked_by(&self) -> &[Picker] {
        pickers_of(&self.run)
    }
}

/// How points along one crossing are put in order.
enum HowItRuns {
    /// It runs on for ever, so the order is a comparison on the axis it climbs
    /// fastest along. `forward_climbs_the_axis` says whether `∇S × ∇Q` points up
    /// that axis rather than down it.
    AlongAnAxis {
        axis: usize,
        forward_climbs_the_axis: bool,
    },
    /// It closes, so the order is a cyclic one and needs somewhere to start.
    /// `from` is where [`walk`](rcad_geom::walk) starts, which depends on the
    /// unordered pair of surfaces and on nothing else (§34.4).
    RoundAClosedCurve { from: PredicatePoint },
    /// It runs on for ever **and bends**, so it has no axis to climb and
    /// nowhere to start: a plane cutting a cone in a parabola or in one branch
    /// of a hyperbola. The order is how far along `n × w` a point sits, for the
    /// plane's normal `n` and the cone's axis direction `w`
    /// ([`along_a_bend`] carries the derivation).
    AlongABend {
        plane_normal: [f64; 3],
        cone_axis: [f64; 3],
        /// Whether `∇S × ∇Q` points the way `n × w` does rather than the other
        /// way — the job [`HowItRuns::AlongAnAxis`]'s
        /// `forward_climbs_the_axis` does one variant up.
        forward_runs_with_the_bend: bool,
    },
}

impl HowItRuns {
    fn closes(&self) -> bool {
        matches!(self, HowItRuns::RoundAClosedCurve { .. })
    }

    /// Whether the curve is a straight line.
    ///
    /// **Not the opposite of [`HowItRuns::closes`]**, and the difference is what
    /// the third variant added: a bend neither closes nor runs straight. The
    /// two questions used to have one answer between them, and every place that
    /// wanted *straight* asked for *does not close* because that was the same
    /// thing. It is not any more, and
    /// [`Arrangement::wire_from`] is where the two part company.
    fn runs_straight(&self) -> bool {
        matches!(self, HowItRuns::AlongAnAxis { .. })
    }
}

/// A point of the arrangement.
struct Node {
    point: PredicatePoint,
    /// The input corner this is, when it is one.
    vertex: Option<VertexKey>,
    /// The three surfaces that placed it, in [`meet_of_three_surfaces`]'s own
    /// fixed order, and which of their meeting points this is. `None` for a
    /// corner the input already had.
    made_of: Option<([Surface; 3], u8)>,
    /// The crossings running through it, in increasing order.
    crossings: Vec<usize>,
}

/// A piece of one crossing between two neighbouring nodes on it.
struct Arc {
    crossing: usize,
    /// The two ends, the first one earlier along the crossing's forward
    /// direction.
    ends: [usize; 2],
    /// Which wire of the face walks along here, and which way.
    boundary: Option<Boundary>,
}

/// How the face's own boundary uses an arc.
#[derive(Clone, Copy, PartialEq)]
struct Boundary {
    /// Which wire.
    wire: usize,
    /// Whether the wire walks this arc in the crossing's forward direction.
    runs_forward: bool,
    /// How far round the wire the side this stretch is part of sits. The input
    /// edges are named there rather than copied here, because joining coplanar
    /// input faces can leave a side that is several of them.
    position: usize,
}

/// A way to leave a node: along a crossing, one way or the other.
#[derive(Clone, Copy, PartialEq)]
struct Direction {
    crossing: usize,
    /// `true` for `∇S × ∇Q`, `false` for the other way.
    forward: bool,
}

impl<'f> Arrangement<'f> {
    fn of(
        face: &'f BakedFace,
        cuts: &[Surface],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Self> {
        let mut arrangement = Self {
            face,
            surface: face.surface,
            crossings: Vec::new(),
            nodes: Vec::new(),
            arcs: Vec::new(),
            drawn_in_full: Vec::new(),
            cut_crossings: Vec::new(),
            crossing_of_edge: Vec::new(),
            node_of_corner: Vec::new(),
            node_of_the_end: Vec::new(),
            leaving: Vec::new(),
            cycle_of: Vec::new(),
            cycles: Vec::new(),
            winding_of_cycle: Vec::new(),
            cell_of: Vec::new(),
            cells: Vec::new(),
            winding: Vec::new(),
        };
        arrangement.draw_the_crossings(cuts, policy, log)?;
        arrangement.place_the_corners(policy, log)?;
        arrangement.place_the_meets(policy, log)?;
        arrangement.start_every_closed_crossing_that_has_no_node(log)?;
        arrangement.work_out_how_each_crossing_runs(log)?;
        arrangement.merge_points_that_are_one_point(policy, log)?;
        arrangement.cut_the_crossings_into_arcs(policy, log)?;
        arrangement.walk_the_cycles(policy, log)?;
        arrangement.count_the_windings_inside_each_lump();
        arrangement.group_the_cycles_into_cells(policy, log)?;
        arrangement.count_the_windings()?;
        Ok(arrangement)
    }

    /// The pieces: the cells the face's boundary winds once round.
    ///
    /// # Panics
    ///
    /// If a piece has no sides at all, or if a flat piece has fewer than three
    /// straight ones. Two straight lines enclose no area, so a piece like that
    /// is a zero-area face, which §17.4 prevents rather than filters: reaching
    /// here means the splitter is wrong.
    fn pieces(&self) -> Result<Vec<Piece>> {
        if self.arcs.is_empty() {
            return Ok(vec![self.the_whole_surface()?]);
        }
        (0..self.cells.len())
            .filter(|&cell| self.winding[cell] == Some(1))
            .map(|cell| self.piece_from(cell))
            .collect()
    }

    /// The piece a face with nothing drawn on it at all is: the whole surface,
    /// cut open along a seam of its own.
    ///
    /// **§38.1, and the wrong answer it closes.** A ball's skin is one face
    /// whose one wire is a seam, and a seam is left out at the door (`bake`'s
    /// header), so a ball nothing cuts arrives here with no boundary and no cut
    /// — no curve, no node, no cell, and therefore no piece. `fuse` handed that
    /// back as `Ok` and nothing at all, which is a solid the caller passed in
    /// silently dropped.
    ///
    /// The seam the answer carries is **not the input's**. The answer's face is
    /// written with no placement of its own, so its two coordinates are read off
    /// the surface standing where it is — and a ball has no frame of its own,
    /// so its poles are `centre ± (0, 0, r)` in the world's axes and its
    /// longitude runs from `+x` however the input was turned
    /// ([`Sphere::point_at`](rcad_geom::Sphere::point_at)). A seam read back off
    /// a turned input would name a meridian that is not one of *this* surface's,
    /// which is the same mistake as re-using a corner without re-deriving it
    /// (§17.1).
    ///
    /// # Errors
    ///
    /// [`Error::ACurvedFaceGoesRightRoundWithNoSeam`] when the surface does not
    /// pinch to a point, so there is nowhere for a seam to end. Only a ball
    /// does, and only a ball is a closed surface: a plane and a pipe both run on
    /// for ever, so a face on one of them with nothing trimming it is unbounded
    /// and no answer exists. Nothing in the public API builds one — a profile
    /// has an outline, a bar has two cap circles, and a face this fuse cuts is
    /// bounded by what cut it — but a model read from a file may, and §16.11
    /// says a caller's bytes may not panic.
    ///
    /// **An empty arrangement means an empty boundary and nothing else.** A
    /// crossing carrying a boundary edge is never dropped for missing the face —
    /// [`Arrangement::work_out_how_each_crossing_runs`] only drops one that is
    /// `only_a_cut` — and a boundary edge with two distinct corners leaves at
    /// least one step to become an arc, while one with the same corner twice
    /// closes and leaves a whole curve. So a face with any baked wire in it has
    /// an arc, and reaching here means every wire was a seam.
    fn the_whole_surface(&self) -> Result<Piece> {
        let Surface::Sphere(ball) = self.surface else {
            return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam {
                face: self.face.keys[0],
            });
        };
        let meridian = Surface::Plane(
            Plane::from_point_and_normal(ball.centre(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal"),
        );
        let pole = |at_the_top| MeetPoint::APoleOfAClosedSurface {
            surface: self.surface,
            at_the_top,
            point: where_a_ball_pinches(&ball, at_the_top),
        };
        // **`material_side` is not about material here**, and a seam is the one
        // place it is not: the whole surface is on both sides of the meridian,
        // so there is no side to name. What it is read for downstream is which
        // of the meridian's two halves this seam is — `which_arc_this_is` turns
        // it into `∓(∇S × ∇T)`, and at the top pole `∇S` is straight up and
        // `∇T` is `+y`, so `Pos` is the `+x` half. That is the half
        // `Sphere::point_at(0, ·)` runs along, which is the meridian the
        // surface's own coordinates are cut open at.
        let along = |ends: [MeetPoint; 2]| FaceMeet {
            carried_by: [self.surface, meridian],
            ends,
            material_side: Decision::Pos,
            along: SmallVec::new(),
            approximation: None,
        };
        Ok(Piece {
            ancestors: self.face.keys.clone(),
            surface: self.surface,
            orientation: self.face.orientation,
            wires: vec![vec![
                along([pole(true), pole(false)]),
                along([pole(false), pole(true)]),
            ]],
            is_the_whole_surface: true,
        })
    }

    /// One crossing per curve that matters to this face, with the ones that are
    /// really the same curve folded together.
    fn draw_the_crossings(
        &mut self,
        cuts: &[Surface],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        for wire in 0..self.face.wires.len() {
            let mut along = Vec::new();
            for position in 0..self.face.wires[wire].len() {
                let edge = &self.face.wires[wire][position];
                let (across, at) = (edge.across, edge.start.point);
                let crossing = self
                    .crossing_for(across, false, Some(at), policy, log)?
                    .ok_or(Error::AFaceMeetsItsNeighbourNowhere {
                        face: self.face.keys[0],
                    })?;
                along.push(crossing);
            }
            self.crossing_of_edge.push(along);
        }
        for &cut in cuts {
            let crossing = self.crossing_for(cut, true, None, policy, log)?;
            self.cut_crossings.push(crossing);
        }
        Ok(())
    }

    /// The crossing `surface ∩ partner`, adding it if it is not there yet.
    ///
    /// `None` when the pair names no curve at all — two parallel planes, a ball
    /// clear of the face's own surface.
    ///
    /// A pair can name **two** curves: a plane down the inside of a pipe cuts
    /// two straight lines, and a ball crossing a pipe off-axis leaves two loops.
    /// When it does, `through` is what says which of them is meant: a boundary
    /// edge runs along the one its own corner lies on. A cut surface means all
    /// of them, and hands over `None`.
    ///
    /// # Errors
    ///
    /// [`Error::CannotTellWhichCurveOfACrossingAFaceRunsAlong`] when the point
    /// handed over is picked by more than one of the curves, or by none. A
    /// picker's [`Decision::Undecidable`] rules out nothing (§33.8), so an
    /// ambiguous answer is a refusal and never a guess.
    ///
    /// [`Error::AFaceMeetsItsNeighbourOnItsOwnSurface`] when the two are one
    /// surface. Two faces on one surface with an edge between them carry no
    /// curve for that edge to run along; the joiner in
    /// [`bake`](super::bake) takes such a pair apart before it gets here, and a
    /// pair it does not reach is refused rather than arranged wrongly.
    fn crossing_for(
        &mut self,
        partner: Surface,
        cuts: bool,
        through: Option<PredicatePoint>,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Option<usize>> {
        let runs = match intersect(&self.surface, &partner, policy, log)? {
            SurfaceMeet::Curves(runs) => runs,
            SurfaceMeet::Nothing | SurfaceMeet::TouchAtOnePoint { .. } => return Ok(None),
            SurfaceMeet::SameSurface => {
                return Err(Error::AFaceMeetsItsNeighbourOnItsOwnSurface {
                    face: self.face.keys[0],
                });
            }
            // A sixth shape two surfaces can meet in is `rcad-geom` growing a
            // word, not a broken invariant here and not bad input: the two
            // solids are as sound as they were. So this refuses by name rather
            // than crashing a fuse over well-formed solids, and the name is the
            // true one — a shape this arrangement has no order along.
            //
            // [`SurfaceMeet`] is `#[non_exhaustive]` and belongs to another
            // crate, so a sixth shape arriving here is not a compile error.
            _ => return Err(Error::ACrossingThisCannotBeOrderedAlong),
        };

        let mut wanted = runs;
        if let Some(at) = through {
            wanted.retain(|run| nothing_rules_the_point_out(pickers_of(run), at));
            if wanted.len() != 1 {
                return Err(Error::CannotTellWhichCurveOfACrossingAFaceRunsAlong {
                    face: self.face.keys[0],
                });
            }
        }

        let mut first = None;
        for run in wanted {
            let index = match self.crossing_like(&partner, pickers_of(&run), policy, log)? {
                Some(already) => {
                    self.crossings[already].cuts |= cuts;
                    already
                }
                None => {
                    self.crossings.push(Crossing {
                        partner,
                        run,
                        // Filled in once there is a node to read it at.
                        order: HowItRuns::AlongAnAxis {
                            axis: 0,
                            forward_climbs_the_axis: true,
                        },
                        cuts,
                        reaches_the_face: true,
                    });
                    self.crossings.len() - 1
                }
            };
            first = first.or(Some(index));
        }
        Ok(first)
    }

    /// Which crossing already drawn is this same curve, if any.
    ///
    /// Two *different* surfaces can cut one face in one curve: two planes
    /// through the same line, or two spheres through the same circle. All
    /// planes, and it is the rank of a three-by-four matrix of stored
    /// coefficients, which is four exact determinants
    /// ([`relation_of_two_lines`]). Otherwise the question is whether the two
    /// partners are one surface, which [`intersect`] answers (§31.2 row 7).
    fn crossing_like(
        &mut self,
        partner: &Surface,
        picked_by: &[Picker],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Option<usize>> {
        for index in 0..self.crossings.len() {
            let mine = self.crossings[index].partner;
            let same = match (self.surface.as_plane(), mine.as_plane(), partner.as_plane()) {
                (Some(here), Some(one), Some(other)) => {
                    matches!(
                        relation_of_two_lines(here, one, other, policy, log)?,
                        LineRelation::Same
                    )
                }
                _ => {
                    mine == *partner
                        || matches!(
                            intersect(&mine, partner, policy, log)?,
                            SurfaceMeet::SameSurface
                        )
                }
            };
            if same && self.crossings[index].picked_by() == picked_by {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    /// One node per corner of the face, read out of the baked solid.
    ///
    /// Which crossings run through a corner is mostly a question about the
    /// model: the two edges either side of it, and, where more faces meet there,
    /// every other surface the corner already knows about. Only a cut has to be
    /// *asked*, and asking is [`side_of_surface`].
    fn place_the_corners(&mut self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        for wire in 0..self.face.wires.len() {
            let mut corners = Vec::new();
            let count = self.face.wires[wire].len();
            for position in 0..count {
                let mut through = vec![self.crossing_of_edge[wire][position]];
                let at = self.face.wires[wire][position].start.point;
                let around = self.face.wires[wire][position].start.around.clone();
                for index in 0..self.crossings.len() {
                    if through.contains(&index) {
                        continue;
                    }
                    let known_from_the_model = around.contains(&self.crossings[index].partner);
                    if known_from_the_model || self.passes_through(index, at, policy, log)? {
                        through.push(index);
                    }
                }
                through.sort_unstable();
                through.dedup();
                corners.push(self.nodes.len());
                self.nodes.push(Node {
                    point: at,
                    vertex: Some(self.face.wires[wire][position].start.vertex),
                    made_of: None,
                    crossings: through,
                });
            }
            self.node_of_corner.push(corners);
        }
        self.node_of_the_end = self
            .face
            .wires
            .iter()
            .map(|wire| {
                wire.iter()
                    .map(|side| {
                        self.nodes
                            .iter()
                            .position(|node| node.vertex == Some(side.end.vertex))
                            .expect("the corner an edge stops at is a corner of this face")
                    })
                    .collect()
            })
            .collect();
        Ok(())
    }

    /// Whether a point is on one crossing: on its partner surface, and picked by
    /// whichever curve of the pair this crossing is.
    fn passes_through(
        &self,
        crossing: usize,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<bool> {
        let partner = self.crossings[crossing].partner;
        let settled = must_decide(
            side_of_surface(&partner, point),
            name_of_surface(&partner),
            &[point],
            Measures::Length,
            policy,
            log,
        )?;
        Ok(settled == Decision::Zero
            && nothing_rules_the_point_out(self.crossings[crossing].picked_by(), point))
    }

    /// One node per place two crossings meet, unless a corner already is that
    /// place.
    ///
    /// Three surfaces can meet at two points, and both are nodes.
    /// [`MeetOfThree::MoreThanAPoint`] is an answer and not a refusal (§34.9):
    /// two faces on one plane with a curved surface across them share a whole
    /// curve, and there is no node there rather than a node nothing can place.
    ///
    /// # Not every pair is worth asking about
    ///
    /// A pair of crossings whose two partners share no point at all has no node
    /// between them, and [`meet_of_three_surfaces`] is not the way to find that
    /// out: a plane cut by a ball and by a pipe is one plane short of what it can
    /// solve, so it refuses by name whether or not the ball and the pipe are
    /// anywhere near each other. So when it gives up, ask the cheaper question
    /// first — and only when the answer is *nothing*, which is a proof and not a
    /// distance, is there nothing to place (§48).
    fn place_the_meets(&mut self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        for first in 0..self.crossings.len() {
            for second in (first + 1)..self.crossings.len() {
                if self.crossings[first].partner == self.crossings[second].partner {
                    continue;
                }
                let surfaces = [
                    self.surface,
                    self.crossings[first].partner,
                    self.crossings[second].partner,
                ];
                let asked = three_planes_faced_the_one_way(surfaces);
                let meet =
                    match meet_of_three_surfaces([&asked[0], &asked[1], &asked[2]], policy, log) {
                        Ok(meet) => meet,
                        Err(gave_up_on_it) => {
                            if self.two_crossings_provably_share_nothing(first, second, policy, log)
                            {
                                continue;
                            }
                            return Err(gave_up_on_it.into());
                        }
                    };
                let MeetOfThree::At(places) = meet else {
                    continue;
                };
                for (which, place) in places.into_iter().enumerate() {
                    let place = as_exact_as_it_is(place);
                    // Being *on* both surfaces is what the meet just worked out,
                    // and asking a predicate to confirm it would be a second
                    // opinion about a fact this line already holds — one that a
                    // box straddling the surface cannot prove and would refuse.
                    // What is still open is which curve of each pair it is on,
                    // and only a pair naming more than one has anything to say.
                    if !nothing_rules_the_point_out(self.crossings[first].picked_by(), place)
                        || !nothing_rules_the_point_out(self.crossings[second].picked_by(), place)
                    {
                        continue;
                    }
                    // Only a node that is *this* place is already this place.
                    // Asking whether any node is on both crossings was right
                    // while three planes met once; three surfaces meet twice,
                    // and it threw the second one away — which left a line
                    // crossing a closed curve entering it and never coming out.
                    if self.nodes.iter().any(|node| {
                        node.crossings.contains(&first)
                            && node.crossings.contains(&second)
                            && boxes_overlap(node.point, place)
                    }) {
                        continue;
                    }
                    let mut sorted = surfaces;
                    sorted.sort_by(|one, other| {
                        name_of_surface_bits(one).cmp(&name_of_surface_bits(other))
                    });
                    let which = u8::try_from(which).expect("three surfaces meet at most twice");
                    self.nodes.push(Node {
                        point: place,
                        vertex: None,
                        made_of: Some((sorted, which)),
                        crossings: vec![first, second],
                    });
                }
            }
        }
        Ok(())
    }

    /// Whether the two crossings can be shown to have no point in common.
    ///
    /// A node of this pair would be a point of the face's own surface lying on
    /// both partners, so a pair of partners with nothing in common has no node
    /// between them anywhere — not on this face, not off it, not for any policy.
    /// The face's surface is not asked about: it crosses each partner by
    /// construction, which is why there is a crossing at all, so the partners are
    /// the only pair of the three that can share nothing.
    ///
    /// **The proof is [`SurfaceMeet::Nothing`] and nothing else.**
    /// [`SurfaceMeet::TouchAtOnePoint`] is a point the third surface may well run
    /// through, and every other answer names a whole curve. There is no distance
    /// here and no threshold: a ball a millionth of a unit clear of a pipe is
    /// skipped and a ball resting against one is not.
    ///
    /// An `Err` from [`intersect`] is *no proof* rather than a second failure.
    /// The caller has a refusal in its hand already and reports that one, so
    /// nothing is swallowed: asking a question that did not answer leaves the
    /// answer exactly where it was.
    fn two_crossings_provably_share_nothing(
        &self,
        first: usize,
        second: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> bool {
        let (one, other) = (
            self.crossings[first].partner,
            self.crossings[second].partner,
        );
        matches!(
            intersect(&one, &other, policy, log),
            Ok(SurfaceMeet::Nothing)
        )
    }

    /// A closed crossing with no node on it still needs one.
    ///
    /// A wire needs a coedge, an edge needs two ends, and a wire that closes on
    /// one edge needs both ends to be one vertex. Where the vertex goes must
    /// depend on the two surfaces and on nothing else, or `fuse(A, B)` and
    /// `fuse(B, A)` disagree — so it goes where
    /// [`walk`](rcad_geom::walk) starts, which §34.4 made a function of the
    /// unordered pair (§31.3).
    fn start_every_closed_crossing_that_has_no_node(
        &mut self,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        for crossing in 0..self.crossings.len() {
            if self
                .nodes
                .iter()
                .any(|node| node.crossings.contains(&crossing))
            {
                continue;
            }
            let partner = self.crossings[crossing].partner;
            let Ok(from) = where_a_closed_crossing_starts(
                &self.crossings[crossing].run,
                [&self.surface, &partner],
                log,
            ) else {
                continue;
            };
            // **A box and not the triple it came back as.** Where a closed
            // crossing starts is a *walked* point — the marcher's own seed,
            // settled onto both surfaces — and §34 says in as many words that
            // being a point rather than a box is not a promise it is exact. Read
            // as exact it becomes a corner of a curved face that the sweep can
            // certify is off that face's own surface, which is §15.5's mistake
            // arriving by §27.7's road for the third time in this milestone.
            let point = where_the_crossing_starts(&self.surface, &partner, from);
            self.nodes.push(Node {
                point,
                vertex: None,
                made_of: None,
                crossings: vec![crossing],
            });
        }
        Ok(())
    }

    /// How points along each crossing are ordered, now that there is a node on
    /// it to read the direction at.
    ///
    /// A **cut** every node of which is provably clear of the box round the face
    /// cuts nothing off it, and is dropped rather than ordered. That is not
    /// keenness: a curve that crossed the face's boundary would share a node
    /// with one of the crossings the boundary runs along, and every meet of
    /// every pair has been worked out by now — so a cut with no node inside the
    /// box is a cut wholly outside the face, and how points run along it is a
    /// question about nothing.
    ///
    /// **A crossing the face's own boundary runs along is never dropped**,
    /// however far its nodes are. Its arcs *are* the rim, and dropping one takes
    /// a side off the face — measured, on the planar suite, and it is the whole
    /// of what the word "cut" is doing in the paragraph above.
    ///
    /// It is worth saying because of what it buys. A block wall a hair off
    /// parallel to a bar's axis cuts that bar's cylinder in an ellipse ten
    /// quadrillion across, and three points on one are nearly in a line; the
    /// same wall exactly parallel to it cuts two straight lines whose direction
    /// has to be read off a gradient at a box that far out. Between them, those
    /// two were the whole refusal budget of a turned curved pair, and dropping
    /// the curves that reach nothing takes it to none.
    fn work_out_how_each_crossing_runs(&mut self, log: &mut ProvenanceLog) -> Result<()> {
        for crossing in 0..self.crossings.len() {
            let mut on_it = self
                .nodes
                .iter()
                .filter(|node| node.crossings.contains(&crossing))
                .peekable();
            let only_a_cut = !self
                .crossing_of_edge
                .iter()
                .flatten()
                .any(|&along| along == crossing);
            if only_a_cut
                && on_it.peek().is_some()
                && on_it.all(|node| self.face.bounds.provably_misses(node.point))
            {
                self.crossings[crossing].reaches_the_face = false;
                continue;
            }
            let Some(at) = self
                .nodes
                .iter()
                .find(|node| node.crossings.contains(&crossing))
                .map(|node| node.point)
            else {
                continue;
            };
            let partner = self.crossings[crossing].partner;
            let run = self.crossings[crossing].run.clone();
            self.crossings[crossing].order =
                how_a_crossing_runs(&self.surface, &partner, &run, at, log)?;
        }
        Ok(())
    }

    /// Fold together the nodes that are one point.
    ///
    /// Two nodes are compared only where it matters, which is along a crossing
    /// they share — and every coincidence shows up there, because two nodes that
    /// are the same point are both meets of curves through it. Overlapping
    /// enclosures are exactly the coincidence case (§17.4): the policy decides
    /// whether they merge or the operation gives up.
    fn merge_points_that_are_one_point(
        &mut self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        loop {
            let mut same = Sameness::of(self.nodes.len());
            for crossing in 0..self.crossings.len() {
                if !self.crossings[crossing].reaches_the_face {
                    continue;
                }
                let order = self.nodes_along(crossing, policy, log)?;
                for pair in order.windows(2) {
                    if self.order_along(crossing, pair[0], pair[1], policy, log)? == Decision::Zero
                    {
                        same.join(pair[0], pair[1]);
                    }
                }
            }
            if !same.joined_anything() {
                return Ok(());
            }
            self.fold_the_nodes(&mut same)?;
        }
    }

    fn fold_the_nodes(&mut self, same: &mut Sameness) -> Result<()> {
        let mut folded: Vec<Option<usize>> = vec![None; self.nodes.len()];
        let mut nodes: Vec<Node> = Vec::new();
        for index in 0..self.nodes.len() {
            let leader = same.leader(index);
            match folded[leader] {
                None => {
                    folded[leader] = Some(nodes.len());
                    folded[index] = Some(nodes.len());
                    nodes.push(Node {
                        point: self.nodes[index].point,
                        vertex: self.nodes[index].vertex,
                        made_of: self.nodes[index].made_of,
                        crossings: self.nodes[index].crossings.clone(),
                    });
                }
                Some(into) => {
                    folded[index] = Some(into);
                    let mine = &self.nodes[index];
                    // A corner the input already had wins, whatever its box is:
                    // it is the point the faces around it agree on, and the
                    // neighbouring faces will emit the same one.
                    let takes_over = match (mine.vertex, nodes[into].vertex) {
                        (Some(mine), Some(theirs)) if mine != theirs => {
                            return Err(Invalid::EdgeHasNoLength.into());
                        }
                        (Some(_), Some(_)) => false,
                        (Some(_), None) => true,
                        (None, Some(_)) => false,
                        (None, None) => mine.point.uncertainty() < nodes[into].point.uncertainty(),
                    };
                    if takes_over {
                        nodes[into].vertex = mine.vertex;
                        nodes[into].point = mine.point;
                        nodes[into].made_of = mine.made_of;
                    }
                    let crossings = mine.crossings.clone();
                    nodes[into].crossings.extend(crossings);
                    nodes[into].crossings.sort_unstable();
                    nodes[into].crossings.dedup();
                }
            }
        }
        for corners in self
            .node_of_corner
            .iter_mut()
            .chain(&mut self.node_of_the_end)
        {
            for corner in corners.iter_mut() {
                *corner = folded[*corner].expect("every node lands somewhere");
            }
        }
        self.nodes = nodes;
        Ok(())
    }

    /// The nodes on one crossing, in the order they run along it.
    ///
    /// Sorted on something that settles nothing on its own — the low end of each
    /// enclosure along the dominant axis for a straight crossing, the low end of
    /// how far along `n × w` it reaches for a bend, and the cyclic order from
    /// the crossing's own start for a closed one. Every neighbouring pair is
    /// then *proved* to be in that order or merged, and proved order between
    /// neighbours is order all the way along.
    fn nodes_along(
        &self,
        crossing: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<usize>> {
        let mut order: Vec<usize> = (0..self.nodes.len())
            .filter(|&node| self.nodes[node].crossings.contains(&crossing))
            .collect();
        match self.crossings[crossing].order {
            HowItRuns::AlongAnAxis {
                axis,
                forward_climbs_the_axis,
            } => {
                order.sort_by(|&one, &other| {
                    let low = |node: usize| self.nodes[node].point.enclosure()[axis].lo();
                    low(one).total_cmp(&low(other))
                });
                if !forward_climbs_the_axis {
                    order.reverse();
                }
            }
            HowItRuns::AlongABend {
                plane_normal,
                cone_axis,
                forward_runs_with_the_bend,
            } => {
                let across = across_a_bend(plane_normal, cone_axis);
                order.sort_by(|&one, &other| {
                    let low = |node: usize| dot_of(self.nodes[node].point.enclosure(), across).lo();
                    low(one).total_cmp(&low(other))
                });
                if !forward_runs_with_the_bend {
                    order.reverse();
                }
            }
            HowItRuns::RoundAClosedCurve { .. } => {
                // A cyclic order is not a total one until somewhere is called
                // first, and the crossing's own start is that somewhere. Sorting
                // by insertion keeps the number of comparisons finite and every
                // one of them is asked of the same three points whichever face
                // is asking.
                let mut sorted: Vec<usize> = Vec::with_capacity(order.len());
                for node in order {
                    let mut place = sorted.len();
                    for (index, &placed) in sorted.iter().enumerate() {
                        if self.order_along(crossing, node, placed, policy, log)? == Decision::Pos {
                            place = index;
                            break;
                        }
                    }
                    sorted.insert(place, node);
                }
                order = sorted;
            }
        }
        Ok(order)
    }

    /// The same, without asking a policy: [`Decision::Undecidable`] comes back
    /// as `None` for a caller that has another route to try.
    fn which_comes_first(&self, crossing: usize, first: usize, second: usize) -> Option<Decision> {
        let (one, other) = (self.nodes[first].point, self.nodes[second].point);
        let decided = match self.crossings[crossing].order {
            HowItRuns::AlongAnAxis {
                axis,
                forward_climbs_the_axis,
            } => {
                let mut apart = other.enclosure()[axis] - one.enclosure()[axis];
                if !forward_climbs_the_axis {
                    apart = -apart;
                }
                apart.sign()
            }
            HowItRuns::AlongABend {
                plane_normal,
                cone_axis,
                forward_runs_with_the_bend,
            } => along_a_bend(
                plane_normal,
                cone_axis,
                forward_runs_with_the_bend,
                one,
                other,
            ),
            HowItRuns::RoundAClosedCurve { from } => {
                let partner = self.crossings[crossing].partner;
                round_a_closed_crossing([&self.surface, &partner], from, one, other).ok()?
            }
        };
        matches!(decided, Decision::Pos | Decision::Neg | Decision::Zero).then_some(decided)
    }

    /// Which of two nodes on a crossing comes first along it: [`Decision::Pos`]
    /// when `first` does, [`Decision::Zero`] when they are the same point.
    fn order_along(
        &self,
        crossing: usize,
        first: usize,
        second: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Decision> {
        let (one, other) = (self.nodes[first].point, self.nodes[second].point);
        match self.crossings[crossing].order {
            HowItRuns::AlongAnAxis {
                axis,
                forward_climbs_the_axis,
            } => {
                let mut apart = other.enclosure()[axis] - one.enclosure()[axis];
                if !forward_climbs_the_axis {
                    apart = -apart;
                }
                must_decide(
                    apart.sign(),
                    ALONG_A_CROSSING,
                    &[one, other],
                    Measures::Length,
                    policy,
                    log,
                )
            }
            HowItRuns::AlongABend {
                plane_normal,
                cone_axis,
                forward_runs_with_the_bend,
            } => must_decide(
                along_a_bend(
                    plane_normal,
                    cone_axis,
                    forward_runs_with_the_bend,
                    one,
                    other,
                ),
                ALONG_A_CROSSING,
                &[one, other],
                Measures::Volume,
                policy,
                log,
            ),
            HowItRuns::RoundAClosedCurve { from } => {
                let partner = self.crossings[crossing].partner;
                let decided = round_a_closed_crossing([&self.surface, &partner], from, one, other)?;
                must_decide(
                    decided,
                    ALONG_A_CROSSING,
                    &[one, other],
                    Measures::Volume,
                    policy,
                    log,
                )
            }
        }
    }

    /// Cut every crossing into the arcs between neighbouring nodes on it, and
    /// throw away the ones that carry nothing.
    ///
    /// A **closed** crossing wraps: the last node runs on into the first, so `n`
    /// nodes leave `n` arcs rather than `n − 1`, and one node on its own leaves
    /// one arc that starts and ends there. That is the whole of the difference,
    /// and it is the difference a cap circle and a round hole are made of.
    fn cut_the_crossings_into_arcs(
        &mut self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        self.drawn_in_full = (0..self.crossings.len())
            .map(|crossing| {
                self.crossings[crossing].order.closes() && self.crossings[crossing].reaches_the_face
            })
            .collect();
        for crossing in 0..self.crossings.len() {
            if !self.crossings[crossing].reaches_the_face {
                continue;
            }
            let order = self.nodes_along(crossing, policy, log)?;
            let closes = self.crossings[crossing].order.closes();
            let steps = if closes {
                order.len()
            } else {
                order.len().saturating_sub(1)
            };
            let mut boundary: Vec<Option<Boundary>> = vec![None; steps];
            let mut present = vec![self.crossings[crossing].cuts; steps];

            for wire in 0..self.face.wires.len() {
                let count = self.face.wires[wire].len();
                for position in 0..count {
                    if self.crossing_of_edge[wire][position] != crossing {
                        continue;
                    }
                    let from = at_in(&order, self.node_of_corner[wire][position]);
                    let to = at_in(&order, self.node_of_the_end[wire][position]);
                    let walked = self.arcs_an_edge_covers(crossing, &order, from, to);
                    for (step, runs_forward) in walked {
                        assert!(
                            boundary[step].is_none(),
                            "two edges of one face run along the same stretch of one crossing"
                        );
                        boundary[step] = Some(Boundary {
                            wire,
                            runs_forward,
                            position,
                        });
                        present[step] = true;
                    }
                }
            }

            for step in 0..steps {
                if present[step] {
                    self.arcs.push(Arc {
                        crossing,
                        ends: [order[step], order[(step + 1) % order.len()]],
                        boundary: boundary[step],
                    });
                } else {
                    self.drawn_in_full[crossing] = false;
                }
            }
        }
        self.drop_the_lumps_of_straight_cuts_alone();
        self.join_arcs_that_meet_at_nothing();
        Ok(())
    }

    /// Throw away lumps of the drawing made of straight cut arcs alone.
    ///
    /// A cut that runs on for ever and passes through the face crosses its
    /// boundary to get there, and a crossing is a node the boundary shares — so
    /// a lump of endless cuts with no boundary arc in it never touches the
    /// inside of the face. It bounds nothing that gets kept and it winds nothing
    /// round anything, so it can go.
    ///
    /// **The word "endless" is the whole of what this lane changed here**, and
    /// leaving it out was a straight-line assumption sitting inside a name that
    /// said nothing about straight lines. A cut that *closes* — the circle a
    /// bar's wall cuts in a block's face — lies wholly inside the face and
    /// touches no boundary at all, and dropping it would lose the hole.
    fn drop_the_lumps_of_straight_cuts_alone(&mut self) {
        let mut same = Sameness::of(self.nodes.len());
        for arc in &self.arcs {
            same.join(arc.ends[0], arc.ends[1]);
        }
        let keep: Vec<usize> = self
            .arcs
            .iter()
            .filter(|arc| arc.boundary.is_some() || self.crossings[arc.crossing].order.closes())
            .map(|arc| same.leader(arc.ends[0]))
            .collect();
        self.arcs
            .retain(|arc| keep.contains(&same.leader(arc.ends[0])));
    }

    /// Which arcs of a crossing one boundary edge covers, and which way round it
    /// walks each.
    ///
    /// A **closed** edge starts and ends at the same corner and covers the whole
    /// curve, so it walks every arc; a `∇S × ∇Q` direction is the crossing's own
    /// and which way the wire runs along it is the direction that keeps the
    /// face's material on the left. §31.3 settles that rule and §30.8 item 10 is
    /// what it closes: `(∇S × ∇T)` crossed with the surface normal always points
    /// *into* the partner, so a wire walks a closed edge forward exactly when
    /// the face's material lies on the partner's negative side — which is what
    /// the corner off that partner already says, and what
    /// [`the_side_this_face_lies_on`](crate::holds) reads.
    fn arcs_an_edge_covers(
        &self,
        crossing: usize,
        order: &[usize],
        from: usize,
        to: usize,
    ) -> Vec<(usize, bool)> {
        if from != to {
            let runs_forward = from < to;
            return (from.min(to)..from.max(to))
                .map(|step| (step, runs_forward))
                .collect();
        }
        assert!(
            self.crossings[crossing].order.closes(),
            "an edge that starts and ends at one corner runs right round a closed curve"
        );
        let runs_forward = self.material_is_left_of_forward(crossing);
        (0..order.len()).map(|step| (step, runs_forward)).collect()
    }

    /// Whether the face's material lies on the left of a closed crossing's
    /// forward direction.
    ///
    /// The identity is `(o × d)·∇Q = −|∇S × ∇Q|²` for `d = ∇S × ∇Q` and
    /// `o = ∇S`, which is Cauchy–Schwarz and holds everywhere with no case
    /// analysis: **left of forward is always the negative side of the partner**.
    /// So the wire walks a closed edge forward exactly when the material is
    /// inside the partner, and that is read off a corner of the face that is off
    /// it.
    ///
    /// Where every corner sits on the partner there is nothing to read and the
    /// answer is the crossing's forward direction — a face whose whole boundary
    /// is one closed curve is the piece of its surface the curve leaves, and
    /// both pieces have the same wire. That is
    /// [`NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`](crate::Error) one layer
    /// down, and it is the case §35.10 hands on.
    fn material_is_left_of_forward(&self, crossing: usize) -> bool {
        let partner = self.crossings[crossing].partner;
        for side in self.face.wires.iter().flatten() {
            if side.start.around.contains(&partner) {
                continue;
            }
            match side_of_surface(&partner, side.start.point) {
                Decision::Neg => return true,
                Decision::Pos => return false,
                Decision::Zero | Decision::Undecidable { .. } | _ => (),
            }
        }
        true
    }

    /// Join two arcs that meet at a node nothing else touches.
    ///
    /// A cut crosses the *curve* of a far edge at a point that is not on the
    /// edge itself, and that crossing is a node with nothing else through it. It
    /// would come out as a corner of a piece where the piece has no corner, so
    /// the two arcs either side of it become one. A node the input already had
    /// is never dropped: the faces around it still have it, and dropping it here
    /// would leave an edge with a corner on one side and none on the other.
    fn join_arcs_that_meet_at_nothing(&mut self) {
        loop {
            let Some((node, first, second)) = self.a_node_that_is_no_corner() else {
                return;
            };
            let other_end = |arc: &Arc| {
                if arc.ends[0] == node {
                    arc.ends[1]
                } else {
                    arc.ends[0]
                }
            };
            let (before, after) = (other_end(&self.arcs[first]), other_end(&self.arcs[second]));
            let ends = if self.arcs[first].ends[1] == node {
                [before, after]
            } else {
                [after, before]
            };
            self.arcs[first].ends = ends;
            self.arcs.remove(second);
        }
    }

    fn a_node_that_is_no_corner(&self) -> Option<(usize, usize, usize)> {
        for node in 0..self.nodes.len() {
            if self.nodes[node].vertex.is_some() {
                continue;
            }
            let touching: Vec<usize> = (0..self.arcs.len())
                .filter(|&arc| self.arcs[arc].ends.contains(&node))
                .collect();
            if touching.len() != 2 {
                continue;
            }
            let (first, second) = (&self.arcs[touching[0]], &self.arcs[touching[1]]);
            // A closed crossing with one node is one arc that starts and ends
            // there, and joining it to itself would leave a curve with no point
            // on it at all.
            if first.crossing == second.crossing
                && first.boundary == second.boundary
                && !self.crossings[first.crossing].order.closes()
            {
                return Some((node, touching[0], touching[1]));
            }
        }
        None
    }

    /// Walk every arc from both sides, keeping the region on the left, and see
    /// what the walks close into.
    fn walk_the_cycles(&mut self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        self.leaving = vec![Vec::new(); self.nodes.len()];
        for half_edge in 0..2 * self.arcs.len() {
            let node = self.from(half_edge);
            self.leaving[node].push(half_edge);
        }
        for node in 0..self.nodes.len() {
            let round_here = core::mem::take(&mut self.leaving[node]);
            self.leaving[node] = self.sorted_anticlockwise(node, &round_here, policy, log)?;
        }

        self.cycle_of = vec![usize::MAX; 2 * self.arcs.len()];
        for start in 0..2 * self.arcs.len() {
            if self.cycle_of[start] != usize::MAX {
                continue;
            }
            let mut chain = Vec::new();
            let mut half_edge = start;
            loop {
                self.cycle_of[half_edge] = self.cycles.len();
                chain.push(half_edge);
                half_edge = self.next_around_the_cell(half_edge);
                if half_edge == start {
                    break;
                }
                assert!(
                    chain.len() <= 2 * self.arcs.len(),
                    "a walk round a cell must come back to where it started"
                );
            }
            self.cycles.push(chain);
        }
        Ok(())
    }

    /// The next half-edge round the cell this one has on its left.
    ///
    /// At the node this half-edge arrives at, the cell carries on along the
    /// half-edge one step *clockwise* from the way back down this arc. That is
    /// what keeps the region on the left all the way round, and it is what makes
    /// a bounded cell come out anticlockwise.
    fn next_around_the_cell(&self, half_edge: usize) -> usize {
        let node = self.to(half_edge);
        let back = half_edge ^ 1;
        let leaving = &self.leaving[node];
        let position = leaving
            .iter()
            .position(|&other| other == back)
            .expect("the way back down an arc leaves the node it arrives at");
        leaving[(position + leaving.len() - 1) % leaving.len()]
    }

    /// The half-edges leaving a node, in anticlockwise order.
    ///
    /// A sign from [`Arrangement::turns_anticlockwise`] compares two directions
    /// only when they are less than half a turn apart, so it is not an order on
    /// its own. Splitting the directions into the two halves of the turn that
    /// starts at the first crossing through the node makes it one: inside a half
    /// nothing is more than half a turn from anything else, so the sign is a
    /// genuine total order there, and the two halves are then laid end to end.
    fn sorted_anticlockwise(
        &self,
        node: usize,
        half_edges: &[usize],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<usize>> {
        if half_edges.len() < 2 {
            return Ok(half_edges.to_vec());
        }
        let reference = Direction {
            crossing: half_edges
                .iter()
                .map(|&half_edge| self.direction(half_edge).crossing)
                .min()
                .expect("the line just above handed back every node with fewer than two ways out"),
            forward: true,
        };

        let mut sorted: Vec<usize> = Vec::with_capacity(half_edges.len());
        for &half_edge in half_edges {
            let mine =
                self.half_of_the_turn(node, reference, self.direction(half_edge), policy, log)?;
            let mut place = sorted.len();
            for (index, &placed) in sorted.iter().enumerate() {
                let theirs =
                    self.half_of_the_turn(node, reference, self.direction(placed), policy, log)?;
                let before = match mine.cmp(&theirs) {
                    Ordering::Less => true,
                    Ordering::Greater => false,
                    Ordering::Equal => {
                        let turn = self.turns_anticlockwise_or_bends(
                            node,
                            self.direction(half_edge),
                            self.direction(placed),
                            policy,
                            log,
                        )?;
                        // Two curves that share their whole tangent *and* bend
                        // the same way. Telling them apart takes a derivative
                        // above the ones taken here. A `3 × 4` block corner on a
                        // bar of radius five reaches it: the block's two walls
                        // cut the bar's along the one ruling `(3, 4, z)`, so the
                        // two arcs are the same line and no derivative ever
                        // parts them (§38.2).
                        if turn == Decision::Zero {
                            return Err(Error::TwoCurvesTouchWithoutCrossing {
                                face: self.face.keys[0],
                            });
                        }
                        turn == Decision::Pos
                    }
                };
                if before {
                    place = index;
                    break;
                }
            }
            sorted.insert(place, half_edge);
        }
        Ok(sorted)
    }

    /// Which half of the turn starting at `reference` a direction is in: `0` for
    /// the half that starts at `reference` itself, `1` for the other.
    ///
    /// **This splits on the tangent alone and never on how a curve bends.** The
    /// halves are `[0, π)` and `[π, 2π)` of *tangent* angle, so a way out
    /// running along the reference is in half zero however it curves, and one
    /// running against it is in half one. Splitting them by how they bend
    /// instead puts two directions half a turn apart into one half, and
    /// [`Arrangement::turns_anticlockwise_or_bends`] answers `Zero` for such a
    /// pair — so the sort refuses two ways out it could have ordered. The
    /// tangent line a wall lays along a bar's cap circle is what found that.
    fn half_of_the_turn(
        &self,
        node: usize,
        reference: Direction,
        direction: Direction,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<u8> {
        match self.turns_anticlockwise(node, reference, direction, policy, log)? {
            Decision::Pos => Ok(0),
            Decision::Neg => Ok(1),
            // The reference's own crossing is the one pair no arithmetic here
            // can ever certify: the determinant has the same gradient in two
            // rows, so it is nothing at all in the reals, while a gradient that
            // is a box makes the interval straddle zero every time (§35.2). Any
            // other pair either certifies or comes back `Undecidable`, so which
            // half this one is in is read off the two `forward` flags instead —
            // the reference's own way starts half zero and its way back starts
            // half one.
            Decision::Zero if direction.crossing == reference.crossing => {
                Ok(u8::from(direction.forward != reference.forward))
            }
            // Two different crossings on one line. Along the reference is half
            // zero, against it is half one.
            Decision::Zero => Ok(u8::from(
                !self.leaves_the_node_the_same_way(node, reference, direction, policy, log)?,
            )),
            other => panic!("a turn came back with a decision nothing here knows: {other:?}"),
        }
    }

    /// Whether `second` is anticlockwise of `first` about the face's own
    /// gradient at this node, by less than half a turn.
    ///
    /// An arc of `S ∩ Q` runs along `±(∇S × ∇Q)`, so for two of them
    /// `(∇S × ∇Q₁) × (∇S × ∇Q₂) = det(∇S, ∇Q₁, ∇Q₂) · ∇S`, using
    /// `(a×b)×(a×c) = a·((a×b)·c)` with `a = ∇S`. Dotting that with `∇S` gives
    /// `det(∇S, ∇Q₁, ∇Q₂) · |∇S|²`, and `|∇S|²` is positive, so the sign of the
    /// turn is the sign of that determinant — with a change of sign for each
    /// direction that runs the other way.
    ///
    /// **Neither step used flatness anywhere** (§31.2 row 9), so this is §17.4's
    /// determinant of three stored normals with a gradient in each normal's
    /// place. For three planes the gradients *are* the stored normals and the
    /// arithmetic is bit for bit what it was.
    ///
    /// A determinant of nothing at all means the three gradients lie in one
    /// plane, so both cross products point along that plane's normal and the two
    /// ways out lie on **one line** — the same way along it, or opposite ways.
    /// Neither is anticlockwise of the other by less than half a turn, so `Zero`
    /// is the honest answer to the question this asks, and the two callers want
    /// different things done about it:
    /// [`Arrangement::half_of_the_turn`] wants which way along the line, and
    /// [`Arrangement::turns_anticlockwise_or_bends`] wants how the two curves
    /// bend.
    fn turns_anticlockwise(
        &self,
        node: usize,
        first: Direction,
        second: Direction,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Decision> {
        // Two ways out along one curve are the same curve, and the turn between
        // them is nothing at all. That is a fact about the drawing and not about
        // the arithmetic: worked out, the determinant has the same gradient in
        // two of its rows, and a gradient that is a *box* — every corner of a
        // placed curved solid is one — makes an interval evaluation of it
        // straddle zero. §35.2 again, arriving one function along.
        if first.crossing == second.crossing {
            return Ok(Decision::Zero);
        }
        let at = self.nodes[node].point;
        let mine = gradient_at(&self.surface, at);
        let one = gradient_at(&self.crossings[first.crossing].partner, at);
        let other = gradient_at(&self.crossings[second.crossing].partner, at);
        let decision = must_decide(
            orient3d_of_points(mine, one, other, PredicatePoint::from([0.0; 3])),
            ORIENT3D,
            &[mine, one, other],
            Measures::Volume,
            policy,
            log,
        )?;
        Ok(if first.forward == second.forward {
            decision
        } else {
            the_other_way(decision)
        })
    }

    /// [`Arrangement::turns_anticlockwise`], and where that says the two lie on
    /// one line, how the two curves bend instead.
    ///
    /// Only for two ways out that [`Arrangement::half_of_the_turn`] has already
    /// put in one half. **That is what makes the fall-through sound**: a half is
    /// half a turn of tangent angle wide and is closed at its start and open at
    /// its end, so no two directions in one half are opposite. The two are
    /// tangent, and the order of two tangent curves is settled one derivative
    /// up.
    ///
    /// `Zero` comes back when the bend ties too, and no derivative taken here
    /// parts two curves that agree on both. It also comes back if the two are
    /// opposite after all, which the paragraph above says cannot happen: that is
    /// a refusal standing in for an assert, because a caller with a policy loose
    /// enough to settle the halves inconsistently should lose its answer rather
    /// than the process.
    fn turns_anticlockwise_or_bends(
        &self,
        node: usize,
        first: Direction,
        second: Direction,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Decision> {
        let turn = self.turns_anticlockwise(node, first, second, policy, log)?;
        if turn != Decision::Zero {
            return Ok(turn);
        }
        if !self.leaves_the_node_the_same_way(node, first, second, policy, log)? {
            return Ok(Decision::Zero);
        }
        self.which_way_two_tangent_curves_bend(node, first, second, policy, log)
    }

    /// Whether two ways out of a node that lie on one line go the same way along
    /// it.
    ///
    /// The two directions are parallel, so the sign of their dot product says
    /// which. Nothing at all means one of them is the zero vector, which is a
    /// cut surface tangent to the face's own here: that crossing has no
    /// direction to compare and nothing about the drawing follows from it.
    ///
    /// # Errors
    ///
    /// [`Error::TwoCurvesTouchWithoutCrossing`] for that last case.
    fn leaves_the_node_the_same_way(
        &self,
        node: usize,
        first: Direction,
        second: Direction,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<bool> {
        let at = self.nodes[node].point;
        let mine = self.way_out_of(node, first);
        let theirs = self.way_out_of(node, second);
        let same_way = must_decide(
            dot_of(mine, theirs).sign(),
            THE_SAME_WAY_ALONG_ONE_LINE,
            &[
                at,
                PredicatePoint::Constructed(mine),
                PredicatePoint::Constructed(theirs),
            ],
            Measures::ManyLengths { how_many: 4 },
            policy,
            log,
        )?;
        if same_way == Decision::Zero {
            return Err(Error::TwoCurvesTouchWithoutCrossing {
                face: self.face.keys[0],
            });
        }
        Ok(same_way == Decision::Pos)
    }

    /// Which of two curves that leave a node along one direction bends further
    /// anticlockwise about the face's own gradient.
    ///
    /// The two tangents agree, so the first-order determinant above is nothing
    /// at all and the order is settled one derivative up. Walk a curve `S ∩ Q`
    /// by arc length and differentiating `S(c(s)) = 0` twice gives
    /// `∇S·k = −(t·H_S·t)` for the unit tangent `t` and the curvature vector
    /// `k`, and the same for `Q`. With `m = ∇S × t` those two, plus `k·t = 0`,
    /// fix how far `k` leans along `m`:
    ///
    /// ```text
    /// k = a·m + b·∇S,   b = −(t·H_S·t)/|∇S|²
    ///                   a = [−(t·H_Q·t) − b(∇Q·∇S)] / (∇Q·m)
    /// ```
    ///
    /// so the lean is `A/B` with `A = −(t·H_Q·t)|∇S|² + (t·H_S·t)(∇Q·∇S)` and
    /// `B = ∇Q·m`. Perturbing the first-order cross product by one step of
    /// curvature, `(t + εk₁) × (t + εk₂) · ∇S = ε(a₂ − a₁)|t|²|∇S|²`, so the
    /// turn is the sign of `A₂/B₂ − A₁/B₁`, which is the sign of
    /// `(A₂B₁ − A₁B₂)·B₁·B₂`.
    ///
    /// **`t` is the same vector for both**, taken from `first`. A curvature
    /// vector does not care which way the curve is walked but `m` does, so
    /// asking each curve about its own tangent would compare two leans measured
    /// against opposite sides. Each `A` carries `t` twice and each `B` once, so
    /// the whole value carries `|t|⁵` as a positive factor and nothing but the
    /// magnitude depends on it: the unit tangent is never formed.
    ///
    /// Working `B` out, `∇Q·(∇S × t) = −c·|∇S × ∇Q|²` for a positive `c`, using
    /// that `t` is along `∇S × ∇Q` for *both* curves here — which is the
    /// tangency and holds only because of it. So `B` is nothing at all exactly
    /// when `∇Q` is along `∇S`, which is the two *surfaces* tangent rather than
    /// the two curves. Then the whole value is nothing at all and the caller
    /// refuses, which is right: this says nothing about a crossing that has no
    /// direction to begin with.
    ///
    /// **Eighteen lengths**, counting a gradient as one the way the determinant
    /// above does: `t` is two, `A` is six, `B` is four, and the value multiplies
    /// an `A·B` by two more `B`s.
    fn which_way_two_tangent_curves_bend(
        &self,
        node: usize,
        first: Direction,
        second: Direction,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Decision> {
        let at = self.nodes[node].point;
        let normal = gradient_at(&self.surface, at);
        let along = self.way_out_of(node, first);
        let sideways = cross_of(normal, PredicatePoint::Constructed(along));
        let normal_squared = dot_of(normal.enclosure(), normal.enclosure());
        let face_bend = bend_along(&self.surface, at, along);

        let lean = |which: Direction| {
            let partner = gradient_at(&self.crossings[which.crossing].partner, at);
            let bend = bend_along(&self.crossings[which.crossing].partner, at, along);
            (
                face_bend * dot_of(partner.enclosure(), normal.enclosure()) - bend * normal_squared,
                dot_of(partner.enclosure(), sideways),
            )
        };
        let (mine, over_mine) = lean(first);
        let (theirs, over_theirs) = lean(second);
        let value = (theirs * over_mine - mine * over_theirs) * over_mine * over_theirs;
        must_decide(
            value.sign(),
            HOW_TWO_TANGENT_CURVES_BEND,
            &[at, normal, PredicatePoint::Constructed(along)],
            Measures::ManyLengths { how_many: 18 },
            policy,
            log,
        )
    }

    /// The direction a half-edge leaves a node in: `±(∇S × ∇Q)` for the crossing
    /// it runs on, the sign being which way along the crossing it goes.
    fn way_out_of(&self, node: usize, direction: Direction) -> [Interval; 3] {
        let at = self.nodes[node].point;
        let run = cross_of(
            gradient_at(&self.surface, at),
            gradient_at(&self.crossings[direction.crossing].partner, at),
        );
        if direction.forward {
            run
        } else {
            run.map(|component| Interval::exactly(0.0) - component)
        }
    }

    /// Which cell of the surface each walk bounds.
    ///
    /// Two walks that share a node are joined by the drawing itself and the walk
    /// already told them apart. What is left is joining up walks in different
    /// lumps, and a lump comes in two shapes.
    ///
    /// **A lump every one of whose curves closes** — a hole a cut left in the
    /// middle of a face, or a cap circle of a bar's wall — is placed by which
    /// side of it everything else lies on. A closed curve cuts the surface into
    /// exactly two, so the side is a sign, and two walks bound one cell exactly
    /// when they agree about every such curve.
    ///
    /// **A lump with a curve that runs on for ever** is placed by following one
    /// of them until it runs into the rest of the drawing, which is what
    /// [`Arrangement::piece_around`] does. That route needs a direction to fire
    /// along and a bounded face to hit, and a closed curve has neither — which
    /// is why there are two and not one.
    fn group_the_cycles_into_cells(
        &mut self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let lump = self.lump_of_each_cycle();
        let mut sides: Vec<Vec<Option<Decision>>> = Vec::with_capacity(self.cycles.len());
        for cycle in 0..self.cycles.len() {
            sides.push(self.which_side_of_everything(cycle, policy, log)?);
        }

        self.say_whether_the_face_holds_each_lump_of_closed_cuts(&lump);

        let mut same = Sameness::of(self.cycles.len());
        for one in 0..self.cycles.len() {
            for other in (one + 1)..self.cycles.len() {
                if lump[one] == lump[other] || self.windings_are_known_to_differ(one, other) {
                    continue;
                }
                let mut agreed = 0;
                let mut differed = false;
                for (mine, theirs) in sides[one].iter().zip(&sides[other]) {
                    match (mine, theirs) {
                        (Some(mine), Some(theirs)) if mine == theirs => agreed += 1,
                        (Some(_), Some(_)) => differed = true,
                        _ => (),
                    }
                }
                if agreed > 0 && !differed {
                    same.join(one, other);
                }
            }
        }
        for which in self.lumps_with_a_curve_that_runs_on(&lump) {
            let on_its_own = (0..self.cycles.len())
                .filter(|&cycle| lump[cycle] == which)
                .all(|cycle| {
                    (0..self.cycles.len())
                        .filter(|&other| lump[other] != which)
                        .all(|other| same.leader(cycle) != same.leader(other))
                });
            if !on_its_own {
                continue;
            }
            let (mine, theirs) = self.piece_around(which, &lump, log)?;
            same.join(mine, theirs);
        }

        self.cell_of = vec![usize::MAX; self.cycles.len()];
        for cycle in 0..self.cycles.len() {
            let leader = same.leader(cycle);
            if self.cell_of[leader] == usize::MAX {
                self.cell_of[leader] = self.cells.len();
                self.cells.push(Vec::new());
            }
            self.cell_of[cycle] = self.cell_of[leader];
            let cell = self.cell_of[cycle];
            self.cells[cell].push(cycle);
        }
        Ok(())
    }

    /// Whether the face's boundary is already known to wind different numbers of
    /// times round the regions two walks bound.
    ///
    /// Two walks that disagree about it are never one cell, whatever else they
    /// agree about — the inside of a face and the outside of it are two places.
    fn windings_are_known_to_differ(&self, one: usize, other: usize) -> bool {
        match (self.winding_of_cycle[one], self.winding_of_cycle[other]) {
            (Some(mine), Some(theirs)) => mine != theirs,
            _ => false,
        }
    }

    /// Say whether the face holds each lump made only of cuts that close.
    ///
    /// A closed cut that touches no boundary arc lies wholly inside the face or
    /// wholly outside it — if it crossed the boundary it would share a node with
    /// it and be the same lump — so one point of it answers for the whole lump,
    /// and every walk the lump bounds is wound the same number of times.
    ///
    /// The count is only run on a **flat** face, and what that leaves out is
    /// worth naming rather than glossing. A curved face this kernel *builds* is
    /// bounded by whole closed curves — a bar's wall by its two cap circles, a
    /// ball's skin by nothing at all — and a closed curve is drawn in full, so
    /// the sign of it places any lump without a count. A flat face is bounded by
    /// straight lines, and the stretch of a line past the edge lying on it is
    /// not drawn, so the sign says the same thing on both sides of the face's
    /// own rim: the count is what tells the inside from the outside there.
    ///
    /// **The first curved face with an open boundary edge falsifies the first
    /// half of that**, and it is a face this fuse cuts and hands back — so lane
    /// 5 of §31.6 makes one. Nothing here is wrong when it arrives: an unknown
    /// winding rules nothing out, and what follows either places the lump some
    /// other way or refuses by name.
    fn say_whether_the_face_holds_each_lump_of_closed_cuts(&mut self, lump: &[usize]) {
        let in_plane = self.surface.as_plane().and_then(InPlane::of);
        let rim: Vec<Vec<PredicatePoint>> = self
            .face
            .wires
            .iter()
            .map(|wire| wire.iter().map(|side| side.start.point).collect())
            .collect();

        let mut lumps: Vec<usize> = lump.to_vec();
        lumps.sort_unstable();
        lumps.dedup();
        for which in lumps {
            let its_walks: Vec<usize> = (0..self.cycles.len())
                .filter(|&cycle| lump[cycle] == which)
                .collect();
            if its_walks
                .iter()
                .any(|&cycle| self.winding_of_cycle[cycle].is_some())
            {
                continue;
            }
            // Every node of the lump is tried and not just the first. The whole
            // lump is inside one cell of the rest, so they all give the same
            // answer — but a node that happens to sit on the line of one of the
            // face's own sides is one the fan cannot decide about, and another
            // node of the same lump is not.
            let mut nodes: Vec<usize> = its_walks
                .iter()
                .flat_map(|&cycle| self.cycles[cycle].clone())
                .map(|half_edge| self.from(half_edge))
                .collect();
            nodes.sort_unstable();
            nodes.dedup();
            // A cut whose whole lump is clear of the box round the face holds
            // none of it, and the box is a proof rather than a hint. That is
            // what settles a plane a hair off parallel to a pipe's axis: the
            // ellipse it cuts is real, and ten quadrillion across, and nothing
            // that far away is worth deciding about more carefully.
            let outside = nodes
                .iter()
                .all(|&node| self.face.bounds.provably_misses(self.nodes[node].point));
            let held = if outside {
                Some(0)
            } else {
                in_plane.as_ref().and_then(|in_plane| {
                    nodes.iter().find_map(|&node| {
                        how_often_the_rim_winds_round(in_plane, &rim, self.nodes[node].point)
                    })
                })
            };
            let Some(held) = held else {
                continue;
            };
            for cycle in its_walks {
                self.winding_of_cycle[cycle] = Some(held);
            }
        }
    }

    /// The lumps of the drawing holding a curve that runs on for ever, other
    /// than the one the face's own first wire is in.
    ///
    /// Those are the ones a sign cannot place: a walk elsewhere on the surface
    /// is on one side of an endless curve or the other, but the stretch of that
    /// curve past its outermost node is not drawn, so the two sides there are
    /// one cell.
    fn lumps_with_a_curve_that_runs_on(&self, lump: &[usize]) -> Vec<usize> {
        let home = self.a_lump_of_the_faces_first_wire();
        let mut found: Vec<usize> = self
            .arcs
            .iter()
            .filter(|arc| !self.crossings[arc.crossing].order.closes())
            .map(|arc| lump[self.cycle_of[2 * self.position_of(arc)]])
            .filter(|&which| Some(which) != home)
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    }

    fn position_of(&self, arc: &Arc) -> usize {
        self.arcs
            .iter()
            .position(|other| core::ptr::eq(other, arc))
            .expect("the only caller borrows the arc straight out of this same list")
    }

    /// Which walk elsewhere in the drawing a lump of endless curves sits inside,
    /// and which of the lump's own walks faces it.
    ///
    /// Follow one of the lump's own curves away from one of its nodes. Nothing
    /// changes which cell of the rest of the drawing we are in until we run into
    /// an arc of it, so the first arc we meet bounds the cell we started in, and
    /// which side of that arc we came from says which walk that is.
    ///
    /// Every node of the lump is tried, not just the first. The whole lump is
    /// inside one cell of the rest of the drawing, so every node of it gives the
    /// same answer, and a corner where every ray runs into the end of an arc is
    /// no reason to give up while another corner has a ray that does not.
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`](crate::Error::Undecidable) when no ray through the
    /// lump gives an answer.
    ///
    /// A ray that cannot be followed — one that runs along an arc, or that
    /// reaches an arc's end, or whose distances cannot be told apart — is
    /// abandoned and another is tried, without a word in the log. Abandoning a
    /// route is not a decision anybody failed to prove, and a record of a way we
    /// did not go would be noise in a log whose emptiness is supposed to mean
    /// something (§5).
    fn piece_around(
        &self,
        lump: usize,
        lump_of: &[usize],
        log: &mut ProvenanceLog,
    ) -> Result<(usize, usize)> {
        let home = self.a_lump_of_the_faces_first_wire();
        let mut corners: Vec<usize> = (0..self.arcs.len())
            .filter(|&arc| lump_of[self.cycle_of[2 * arc]] == lump)
            .flat_map(|arc| self.arcs[arc].ends)
            .collect();
        corners.sort_unstable();
        corners.dedup();
        assert!(!corners.is_empty(), "a lump has an arc in it");

        for &from in &corners {
            for crossing in self.nodes[from].crossings.clone() {
                for forward in [true, false] {
                    if let Some(found) =
                        self.first_arc_met(from, Direction { crossing, forward }, home, lump_of)
                    {
                        return Ok(found);
                    }
                }
            }
        }
        Err(gave_up(
            AROUND_A_HOLE,
            &[self.nodes[corners[0]].point],
            Measures::Length,
            log,
        ))
    }

    /// The walk of `home`'s drawing that a ray from `from` along `direction`
    /// starts in, together with the walk of the ray's own lump that faces it —
    /// or `None` when this ray cannot say.
    fn first_arc_met(
        &self,
        from: usize,
        direction: Direction,
        home: Option<usize>,
        lump_of: &[usize],
    ) -> Option<(usize, usize)> {
        if self.crossings[direction.crossing].order.closes() {
            return None;
        }
        let mut nearest: Option<(usize, usize)> = None;
        for arc in 0..self.arcs.len() {
            // Asked of every arc, the lump's own included: a ray that runs along
            // an arc cannot say which side of it we are on, and one of the
            // lump's own arcs is as much in the way as anybody else's. An arc
            // that only touches the ray at the corner it starts from, and
            // otherwise lies behind, is not in the way at all — and it is the
            // ordinary case, because the corner a ray starts from is a corner of
            // the ring the ray is trying to place.
            if self.arcs[arc].crossing == direction.crossing {
                let not_behind_us = |end: usize| {
                    end != from && self.along_the_ray(direction, from, end) != Some(Decision::Neg)
                };
                if self.arcs[arc].ends.iter().copied().any(not_behind_us) {
                    return None;
                }
                continue;
            }
            if Some(lump_of[self.cycle_of[2 * arc]]) != home {
                continue;
            }
            let at = match self.where_the_ray_crosses(direction.crossing, arc) {
                MetAt::Nowhere => continue,
                // Touching an arc's end says nothing about which side of it we
                // are on — but only where the touch is ahead of us. An arc the
                // ray grazes behind our back is out of the story.
                MetAt::AnEnd(at) => match self.along_the_ray(direction, from, at) {
                    Some(Decision::Neg) => continue,
                    _ => return None,
                },
                MetAt::Between(at) => at,
            };
            match self.along_the_ray(direction, from, at) {
                Some(Decision::Pos) => (),
                Some(Decision::Neg) => continue,
                _ => return None,
            }
            nearest = match nearest {
                None => Some((at, arc)),
                Some((best, best_arc)) => match self.along_the_ray(direction, best, at) {
                    Some(Decision::Neg) => Some((at, arc)),
                    Some(Decision::Pos) => Some((best, best_arc)),
                    _ => return None,
                },
            };
        }
        let (_, arc) = nearest?;
        // An arc's forward half-edge has the negative side of its partner on its
        // left: on the plane `z = 0` the line `x = 0` runs along `z × x = +y`,
        // and walking up `y` with `z` overhead leaves `x < 0` on the left.
        let partner = self.crossings[self.arcs[arc].crossing].partner;
        let half_edge = match side_of_surface(&partner, self.nodes[from].point) {
            Decision::Neg => 2 * arc,
            Decision::Pos => 2 * arc + 1,
            Decision::Zero | Decision::Undecidable { .. } | _ => return None,
        };
        let facing = self.the_walk_of_a_lump_that_is_wound_once(
            lump_of[self.cycle_of
                [2 * (0..self.arcs.len()).find(|&arc| self.arcs[arc].ends.contains(&from))?]],
        )?;
        Some((facing, self.cycle_of[half_edge]))
    }

    /// The walk of a lump that the face's boundary winds once round.
    ///
    /// A hole of the face has a material side and a hollow side, and it is the
    /// material side that becomes a wire of the piece round it. A lump with two
    /// of them, or none, is not a hole of this face and there is nothing to
    /// place.
    fn the_walk_of_a_lump_that_is_wound_once(&self, lump: usize) -> Option<usize> {
        let lump_of = self.lump_of_each_cycle();
        let mut wound = (0..self.cycles.len())
            .filter(|&cycle| lump_of[cycle] == lump && self.winding_of_cycle[cycle] == Some(1));
        let only = wound.next()?;
        wound.next().is_none().then_some(only)
    }

    /// Where the ray's crossing meets an arc.
    ///
    /// Every place two crossings meet already has a node
    /// ([`Arrangement::place_the_meets`]), so nothing is worked out again here.
    fn where_the_ray_crosses(&self, crossing: usize, arc: usize) -> MetAt {
        let other = self.arcs[arc].crossing;
        let Some(at) = (0..self.nodes.len()).find(|&node| {
            self.nodes[node].crossings.contains(&crossing)
                && self.nodes[node].crossings.contains(&other)
        }) else {
            return MetAt::Nowhere;
        };
        let past = |end: usize| self.which_comes_first(other, at, end);
        match (past(self.arcs[arc].ends[0]), past(self.arcs[arc].ends[1])) {
            (Some(Decision::Neg), Some(Decision::Pos))
            | (Some(Decision::Pos), Some(Decision::Neg)) => MetAt::Between(at),
            (Some(Decision::Neg), Some(Decision::Neg))
            | (Some(Decision::Pos), Some(Decision::Pos)) => MetAt::Nowhere,
            _ => MetAt::AnEnd(at),
        }
    }

    /// Whether `to` is ahead of `from` along a ray, without asking a policy.
    ///
    /// A ray that cannot be followed is abandoned and another is tried, so a
    /// decision nobody could prove is a reason to give up on this route and
    /// never a reason to refuse the operation.
    fn along_the_ray(&self, direction: Direction, from: usize, to: usize) -> Option<Decision> {
        let decided = self.which_comes_first(direction.crossing, from, to)?;
        Some(if direction.forward {
            decided
        } else {
            the_other_way(decided)
        })
    }

    /// Which lump of the drawing each walk belongs to.
    fn lump_of_each_cycle(&self) -> Vec<usize> {
        let mut same = Sameness::of(self.nodes.len());
        for arc in &self.arcs {
            same.join(arc.ends[0], arc.ends[1]);
        }
        self.cycles
            .iter()
            .map(|chain| same.leader(self.from(chain[0])))
            .collect()
    }

    /// Which side of every crossing drawn in full one walk lies on, or `None`
    /// where nothing here can say.
    ///
    /// Read off the half-edge where the walk uses that crossing — the region
    /// left of a forward half-edge is the partner's negative side, which is the
    /// same identity [`Arrangement::material_is_left_of_forward`] rests on — and
    /// off the walk's own nodes where it does not. A crossing that does not
    /// touch this lump at all leaves the whole lump on one side of itself, so
    /// any node of the walk answers for all of them.
    fn which_side_of_everything(
        &self,
        cycle: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<Option<Decision>>> {
        let mut sides = vec![None; self.crossings.len()];
        for &half_edge in &self.cycles[cycle] {
            let crossing = self.arcs[half_edge / 2].crossing;
            if !self.drawn_in_full[crossing] {
                continue;
            }
            sides[crossing] = Some(if half_edge.is_multiple_of(2) {
                Decision::Neg
            } else {
                Decision::Pos
            });
        }
        for (crossing, side) in sides.iter_mut().enumerate() {
            if !self.drawn_in_full[crossing] || side.is_some() {
                continue;
            }
            let partner = self.crossings[crossing].partner;
            for &half_edge in &self.cycles[cycle] {
                let at = self.nodes[self.arcs[half_edge / 2].ends[half_edge % 2]].point;
                let found = must_decide(
                    side_of_surface(&partner, at),
                    name_of_surface(&partner),
                    &[at],
                    Measures::Length,
                    policy,
                    log,
                )?;
                if found != Decision::Zero {
                    *side = Some(found);
                    break;
                }
            }
        }
        Ok(sides)
    }

    /// How many times the face's boundary winds round each walk, counting only
    /// what one lump of the drawing can say on its own.
    ///
    /// The face is on the left of its own boundary, so the walk on that side of
    /// a boundary arc is wound once; crossing a boundary arc the other way takes
    /// one off and crossing a cut arc takes nothing. A lump with no boundary arc
    /// in it — a hole a cut left in the middle of a face — has nowhere to start,
    /// and says nothing until it is joined to a lump that has.
    fn count_the_windings_inside_each_lump(&mut self) {
        self.winding_of_cycle = vec![None; self.cycles.len()];
        for arc in 0..self.arcs.len() {
            let Some(boundary) = self.arcs[arc].boundary else {
                continue;
            };
            let material = self.cycle_of[2 * arc + usize::from(!boundary.runs_forward)];
            if self.winding_of_cycle[material].is_some() {
                continue;
            }
            self.winding_of_cycle[material] = Some(1);
            let mut waiting = vec![material];
            while let Some(cycle) = waiting.pop() {
                let known = self.winding_of_cycle[cycle].expect("only settled walks wait");
                for step in 0..self.cycles[cycle].len() {
                    let half_edge = self.cycles[cycle][step];
                    let across = self.cycle_of[half_edge ^ 1];
                    let theirs = known + self.climb_across(half_edge);
                    if self.winding_of_cycle[across].is_none() {
                        self.winding_of_cycle[across] = Some(theirs);
                        waiting.push(across);
                    }
                }
            }
        }
    }

    /// What crossing one half-edge into the walk on its other side adds to the
    /// winding count.
    fn climb_across(&self, half_edge: usize) -> i32 {
        let arc = half_edge / 2;
        match self.arcs[arc].boundary {
            None => 0,
            Some(boundary) => {
                let material = 2 * arc + usize::from(!boundary.runs_forward);
                if material == half_edge { -1 } else { 1 }
            }
        }
    }

    /// How many times the face's boundary winds round each cell.
    ///
    /// One inside the face and nothing outside it. Crossing a boundary arc from
    /// the right of the way its wire runs to the left adds one, because the face
    /// is on the left of its own boundary; crossing a cut arc adds nothing.
    ///
    /// # Panics
    ///
    /// If two ways round to the same cell disagree, or if a cell comes out wound
    /// more than once or less than none. A face that overlaps itself could do
    /// that, and it is not a face.
    fn count_the_windings(&mut self) -> Result<()> {
        self.winding = vec![None; self.cells.len()];
        // A face with no boundary at all is the whole of its surface, and every
        // cell the cuts leave is inside it. That is only a closed surface — a
        // ball's skin, whose one wire is a seam and whose seam is left out — and
        // it is §31.3's "a split curved face may have no outer wire" met at its
        // simplest: there is no wire at all to read one off.
        if self.arcs.iter().all(|arc| arc.boundary.is_none()) {
            self.winding = vec![Some(1); self.cells.len()];
            return Ok(());
        }
        for cycle in 0..self.cycles.len() {
            let Some(known) = self.winding_of_cycle[cycle] else {
                continue;
            };
            let cell = self.cell_of[cycle];
            match self.winding[cell] {
                None => self.winding[cell] = Some(known),
                Some(already) if already == known => (),
                Some(_) => {
                    return Err(Error::CannotTellTheCellsOfAFaceApart {
                        face: self.face.keys[0],
                    });
                }
            }
        }
        for cell in 0..self.cells.len() {
            if self.winding[cell].is_some() {
                self.spread_the_winding(cell)?;
            }
        }
        Ok(())
    }

    fn spread_the_winding(&mut self, from: usize) -> Result<()> {
        let mut waiting = vec![from];
        while let Some(cell) = waiting.pop() {
            let known = self.winding[cell].expect("only settled cells wait");
            for cycle in self.cells[cell].clone() {
                for step in 0..self.cycles[cycle].len() {
                    let half_edge = self.cycles[cycle][step];
                    let across = self.cell_of[self.cycle_of[half_edge ^ 1]];
                    let theirs = known + self.climb_across(half_edge);
                    if !(0..=1).contains(&theirs) {
                        return Err(Error::CannotTellTheCellsOfAFaceApart {
                            face: self.face.keys[0],
                        });
                    }
                    match self.winding[across] {
                        None => {
                            self.winding[across] = Some(theirs);
                            waiting.push(across);
                        }
                        Some(already) if already == theirs => (),
                        Some(_) => {
                            return Err(Error::CannotTellTheCellsOfAFaceApart {
                                face: self.face.keys[0],
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Where the one cut this arrangement was built with runs through the inside
    /// of the face.
    ///
    /// A cut that turned out to be a curve the face's own boundary already runs
    /// along has no cut arcs at all, and the answer is that it goes nowhere near
    /// the inside — which is right: the boundary is not the inside.
    ///
    /// **A crossing that closes or bends has no axis to measure along and comes
    /// back empty.** That makes the self-crossing check say nothing about a
    /// curved pair rather than the wrong thing, and §36 files it as a gap.
    fn stretches_inside(&self) -> Stretches {
        let nothing = Stretches {
            axis: 0,
            spans: Vec::new(),
        };
        let Some(crossing) = self.cut_crossings[0] else {
            return nothing;
        };
        let HowItRuns::AlongAnAxis { axis, .. } = self.crossings[crossing].order else {
            return nothing;
        };
        let spans = (0..self.arcs.len())
            .filter(|&arc| {
                self.arcs[arc].crossing == crossing
                    && self.arcs[arc].boundary.is_none()
                    && self.winding[self.cell_of[self.cycle_of[2 * arc]]] == Some(1)
            })
            .map(|arc| {
                self.arcs[arc]
                    .ends
                    .map(|end| self.nodes[end].point.enclosure()[axis])
            })
            .collect();
        Stretches { axis, spans }
    }

    /// # Panics
    ///
    /// If the piece has no wires, or if a wire made only of straight sides has
    /// fewer than three of them. Two straight lines enclose no area, so a piece
    /// like that is a zero-area face, which §17.4 prevents rather than filters:
    /// reaching here means the splitter is wrong.
    fn piece_from(&self, cell: usize) -> Result<Piece> {
        let mut cycles = self.cells[cell].clone();
        // A flat piece has an outer wire and holes, and `Model::validate` reads
        // the first as the outer one. The outer wire is the walk that keeps the
        // face's own outer boundary, which is the lump the face's first wire is
        // in. On a closed surface neither is outer (§31.3) and this leaves the
        // order the walks were found in.
        let outer = self.a_lump_of_the_faces_first_wire();
        let lump = self.lump_of_each_cycle();
        cycles.sort_by_key(|&cycle| usize::from(Some(lump[cycle]) != outer));

        let wires: Vec<Vec<FaceMeet>> = cycles
            .iter()
            .map(|&cycle| self.wire_from(cycle))
            .collect::<Result<_>>()?;
        assert!(!wires.is_empty(), "a piece of a face has a boundary");
        Ok(Piece {
            ancestors: self.face.keys.clone(),
            surface: self.surface,
            orientation: self.face.orientation,
            wires,
            is_the_whole_surface: false,
        })
    }

    fn a_lump_of_the_faces_first_wire(&self) -> Option<usize> {
        let lump = self.lump_of_each_cycle();
        (0..2 * self.arcs.len())
            .find(|&half_edge| {
                self.arcs[half_edge / 2]
                    .boundary
                    .is_some_and(|boundary| boundary.wire == 0)
            })
            .map(|half_edge| lump[self.cycle_of[half_edge]])
    }

    /// # Panics
    ///
    /// If a wire made only of straight sides has fewer than three of them. Two
    /// straight lines enclose no area. **Two sides one of which bends do**, and
    /// that is the whole reason this asks
    /// [`HowItRuns::runs_straight`] rather than the "does not close" it used to:
    /// a block wall cuts a cone in a hyperbola, and the piece that hyperbola
    /// leaves against the wall's own edge is a wire of two sides.
    fn wire_from(&self, cycle: usize) -> Result<Vec<FaceMeet>> {
        let all_straight = self.cycles[cycle].iter().all(|&half_edge| {
            self.crossings[self.arcs[half_edge / 2].crossing]
                .order
                .runs_straight()
        });
        assert!(
            !all_straight || self.cycles[cycle].len() >= 3,
            "a piece bounded by straight sides has at least three of them, this one has {}",
            self.cycles[cycle].len()
        );
        self.cycles[cycle]
            .iter()
            .map(|&half_edge| {
                let arc = &self.arcs[half_edge / 2];
                Ok(FaceMeet {
                    carried_by: [self.surface, self.crossings[arc.crossing].partner],
                    material_side: if half_edge.is_multiple_of(2) {
                        Decision::Neg
                    } else {
                        Decision::Pos
                    },
                    ends: [
                        self.meet_point(self.from(half_edge)),
                        self.meet_point(self.to(half_edge)),
                    ],
                    along: arc.boundary.map_or_else(SmallVec::new, |boundary| {
                        self.face.wires[boundary.wire][boundary.position]
                            .keys
                            .clone()
                    }),
                    approximation: None,
                })
            })
            .collect()
    }

    fn meet_point(&self, node: usize) -> MeetPoint {
        match (self.nodes[node].vertex, self.nodes[node].made_of) {
            (Some(vertex), _) => MeetPoint::Existing {
                vertex,
                point: self.nodes[node].point,
            },
            (None, Some((surfaces, which))) => MeetPoint::OfThreeSurfaces {
                surfaces,
                which,
                point: self.nodes[node].point,
            },
            (None, None) => MeetPoint::WhereAClosedCurveStarts {
                carried_by: [
                    self.surface,
                    self.crossings[self.nodes[node].crossings[0]].partner,
                ],
                point: self.nodes[node].point,
            },
        }
    }

    fn direction(&self, half_edge: usize) -> Direction {
        Direction {
            crossing: self.arcs[half_edge / 2].crossing,
            forward: half_edge.is_multiple_of(2),
        }
    }

    fn from(&self, half_edge: usize) -> usize {
        self.arcs[half_edge / 2].ends[half_edge % 2]
    }

    fn to(&self, half_edge: usize) -> usize {
        self.arcs[half_edge / 2].ends[1 - half_edge % 2]
    }
}

/// Where a ball's skin pinches to a point, as a box round it.
///
/// A ball has no frame of its own — it is the same ball however it is turned —
/// so its two poles are `centre ± (0, 0, r)` in the world's axes and nowhere
/// else, which is exactly what `Sphere::point_at` reads back.
///
/// The sum rounds, so it is done in intervals and the box collapses onto one
/// triple whenever it does not round. Handing back the rounded triple as an
/// exact one would certify a confident sign about a rounding, which is §34.6's
/// mistake one surface over.
fn where_a_ball_pinches(ball: &Sphere, at_the_top: bool) -> PredicatePoint {
    let centre = ball.centre().coordinates();
    let up = if at_the_top {
        ball.radius()
    } else {
        -ball.radius()
    };
    as_exact_as_it_is(PredicatePoint::Constructed([
        Interval::exactly(centre[0]),
        Interval::exactly(centre[1]),
        Interval::exactly(centre[2]) + Interval::exactly(up),
    ]))
}

/// How many times a face's own rim winds round a point on its surface: one
/// inside the material, zero outside it.
///
/// A half-line runs from the point towards the first corner of the rim and the
/// sides crossing it are counted, `+1` one way round and `-1` the other, so a
/// hole subtracts itself. Which side of the half-line's line each corner falls
/// is read as a **half-open** range — a corner exactly on it counts as below —
/// and that is what leaves no tie to break however many corners the line runs
/// through.
///
/// **Deliberately not the fan `select.rs` uses**, and the reason is one shape:
/// a fan from a corner cannot answer about a point on one of its own chords,
/// and every chord of a square runs through its centre. The circle a bar cuts in
/// the middle of a block's face starts at a point that is often exactly there.
/// The two are the same question and §35.10's last trap already asks for one
/// answer to it; this is a third spelling and lane 5 owes the unification.
///
/// `None` when the point sits on the line of one of the rim's own sides, or when
/// a turn could not be settled. Both mean "ask about another point", which the
/// caller has.
fn how_often_the_rim_winds_round(
    in_plane: &InPlane,
    rim: &[Vec<PredicatePoint>],
    point: PredicatePoint,
) -> Option<i32> {
    let aim = *rim.iter().flatten().next()?;
    let mut winding = 0;
    for (position, ring) in rim.iter().enumerate() {
        let inwards = if position == 0 { 1 } else { -1 };
        for index in 0..ring.len() {
            let (from, to) = (ring[index], ring[(index + 1) % ring.len()]);
            let along = in_plane.turn(from, to, point);
            if !matches!(along, Decision::Pos | Decision::Neg) {
                return None;
            }
            let above = |at: PredicatePoint| {
                // The corner the half-line is aimed at is on it by
                // construction, and the half-open rule counts that as below. It
                // has to be said rather than worked out: the determinant has the
                // same box in two of its rows, and an interval evaluation of one
                // like that straddles zero however far the point is from
                // anything. Every corner of a placed solid is such a box.
                if at == aim {
                    return Some(false);
                }
                match in_plane.turn(point, aim, at) {
                    Decision::Pos => Some(true),
                    Decision::Neg | Decision::Zero => Some(false),
                    Decision::Undecidable { .. } | _ => None,
                }
            };
            winding += inwards
                * match (above(from)?, above(to)?, along) {
                    (false, true, Decision::Pos) => 1,
                    (true, false, Decision::Neg) => -1,
                    _ => 0,
                };
        }
    }
    Some(winding)
}

/// Where a closed crossing starts, as what is really known about it.
///
/// **A box and not the triple it came back as, unless the triple is proved.**
/// Where a closed crossing starts is a *walked* point — the marcher's own seed,
/// settled onto both surfaces — and §34 says in as many words that being a point
/// rather than a box is not a promise it is exact. Read as exact it becomes a
/// corner of a curved face the sweep can certify is *off* that face's own
/// surface, which is §15.5's mistake arriving by §27.7's road.
///
/// A point the two surfaces both certify `Zero` on **is** on both of them, and
/// saying so is a statement about what was proved rather than a rounding — the
/// same sentence [`as_exact_as_it_is`] makes about a box of no width. It matters
/// as much: a circle drawn on the lattice starts at a whole number, and a box
/// round it would turn every sign the arrangement asks about it into a refusal.
///
/// # Panics
///
/// If the walk handed back a seed that is not a place. A [`Point`] carries no
/// promise of being finite — `Point`'s own docs say the types that build
/// geometry out of one check [`Point::is_finite`] and refuse — so the promise
/// has to come from somewhere, and here it is the walk: a seed is marched
/// between two surfaces the model already accepted, and both refuse a
/// coordinate that is not finite when they are built. A seed that is not a
/// place is a bug in the walk, and a box round it is not a box.
fn where_the_crossing_starts(surface: &Surface, partner: &Surface, from: Point) -> PredicatePoint {
    assert!(
        from.is_finite(),
        "a closed crossing is walked between two surfaces built from finite numbers, so where it \
         starts is a real place"
    );
    let exact = PredicatePoint::from(from.coordinates());
    if side_of_surface(surface, exact) == Decision::Zero
        && side_of_surface(partner, exact) == Decision::Zero
    {
        return exact;
    }
    let reach = POINT_ALONG_A_CROSSING_ROUNDOFF * from.largest_coordinate_magnitude().max(1.0);
    PredicatePoint::Constructed(from.coordinates().map(|coordinate| {
        Interval::new(coordinate - reach, coordinate + reach)
            .expect("a finite seed and a positive reach leave a low end below a high one")
    }))
}

/// Whether two boxes have any point in common, so that one may be the other.
///
/// A conservative dedupe and nothing more: two nodes this calls apart are still
/// folded together by [`Arrangement::merge_points_that_are_one_point`] if they
/// turn out to be one point, which is where that decision is taken under a
/// policy.
fn boxes_overlap(one: PredicatePoint, other: PredicatePoint) -> bool {
    let (mine, theirs) = (one.enclosure(), other.enclosure());
    (0..3).all(|axis| mine[axis].lo() <= theirs[axis].hi() && theirs[axis].lo() <= mine[axis].hi())
}

/// Which of two points comes first going round a closed crossing from `from`.
///
/// [`which_comes_first_round_a_crossing`] answers [`Decision::Zero`] when the
/// three points sweep no area, which on a conic means two of them are the same
/// place on it. Two of the three may be **the start**, and then "the same place"
/// is not what it means: a point at the start comes first, by definition of
/// where the counting begins. Reading that as "these are one point" would fold
/// every node of the crossing into the start.
fn round_a_closed_crossing(
    pair: [&Surface; 2],
    from: PredicatePoint,
    one: PredicatePoint,
    other: PredicatePoint,
) -> rcad_geom::Result<Decision> {
    if one == other {
        return Ok(Decision::Zero);
    }
    if one == from {
        return Ok(Decision::Pos);
    }
    if other == from {
        return Ok(Decision::Neg);
    }
    which_comes_first_round_a_crossing(pair, from, one, other)
}

/// What picks one curve of a crossing out of the others, empty when the pair
/// names one curve and there is nothing to pick.
///
/// # Panics
///
/// If the run is a kind this crate has not been taught. Empty is what says "the
/// pair names one curve", and answering it for a kind that carries pickers picks
/// the wrong curve of two rather than refusing — [`EdgeRun`] is
/// `#[non_exhaustive]` and lives in another crate, so a new kind arrives with no
/// compile error and this assert is what notices. A wrong curve is a wrong
/// answer; a crash is not.
fn pickers_of(run: &NamedRun) -> &[Picker] {
    if let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() {
        return picked_by;
    }
    assert!(
        matches!(
            run.run(),
            EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross
        ),
        "a run nothing here can say what picks out: {:?}",
        run.run()
    );
    &[]
}

/// Whether nothing among `picked_by` rules `at` out.
///
/// A picker's [`Decision::Undecidable`] is a real answer and rules out nothing
/// (§33.8), so a point nothing could decide about comes back held by every
/// curve of the pair — and the caller refuses rather than picking the wrong one.
fn nothing_rules_the_point_out(picked_by: &[Picker], at: PredicatePoint) -> bool {
    picked_by
        .iter()
        .all(|picker| picker.side_of_point(at) != Decision::Neg)
}

/// The gradient of a surface at a point, as a box.
///
/// A polynomial in the stored numbers with no normalization in it (§31.2): the
/// normal for a plane, `x − c` for a ball, and `|w|²(x − o) − (w·(x − o))w` for
/// a pipe. For a plane it does not depend on the point at all, which is why the
/// planar arithmetic here is exactly what it was.
fn gradient_at(surface: &Surface, at: PredicatePoint) -> PredicatePoint {
    match surface {
        Surface::Plane(flat) => PredicatePoint::from(flat.normal().components()),
        Surface::Sphere(ball) => {
            let centre = ball.centre().coordinates();
            let box_of_it = at.enclosure();
            PredicatePoint::Constructed(
                [0, 1, 2].map(|axis| box_of_it[axis] - Interval::exactly(centre[axis])),
            )
        }
        Surface::Cylinder(pipe) => {
            let origin = pipe.axis().origin().coordinates();
            let along = pipe.axis().direction().components();
            let box_of_it = at.enclosure();
            let step = [0, 1, 2].map(|axis| box_of_it[axis] - Interval::exactly(origin[axis]));
            let squared = (0..3)
                .map(|axis| Interval::exactly(along[axis]) * Interval::exactly(along[axis]))
                .fold(Interval::exactly(0.0), |so_far, term| so_far + term);
            let shadow = (0..3)
                .map(|axis| Interval::exactly(along[axis]) * step[axis])
                .fold(Interval::exactly(0.0), |so_far, term| so_far + term);
            PredicatePoint::Constructed(
                [0, 1, 2]
                    .map(|axis| squared * step[axis] - shadow * Interval::exactly(along[axis])),
            )
        }
        // `|w|²x − (1 + slope²)(x·w)w` for `x = at − apex`: the pipe's with the
        // shadow term scaled by how far the cone leans.
        Surface::Cone(cone) => {
            let apex = cone.apex().coordinates();
            let along = cone.axis().direction().components();
            let box_of_it = at.enclosure();
            let step = [0, 1, 2].map(|axis| box_of_it[axis] - Interval::exactly(apex[axis]));
            let squared = (0..3)
                .map(|axis| Interval::exactly(along[axis]) * Interval::exactly(along[axis]))
                .fold(Interval::exactly(0.0), |so_far, term| so_far + term);
            let shadow = (0..3)
                .map(|axis| Interval::exactly(along[axis]) * step[axis])
                .fold(Interval::exactly(0.0), |so_far, term| so_far + term);
            let slope = Interval::exactly(cone.slope());
            let steepness = Interval::exactly(1.0) + slope * slope;
            PredicatePoint::Constructed([0, 1, 2].map(|axis| {
                squared * step[axis] - steepness * shadow * Interval::exactly(along[axis])
            }))
        }
        // `(w·w)(x·x + R² − r² − 2R²)x + 2R²(x·w)w` for `x = at − centre`,
        // which is the quartic's own gradient divided by four. Unlike every arm
        // above it depends on `x·x`, because a torus is not a quadric.
        Surface::Torus(ring) => {
            let centre = ring.centre().coordinates();
            let along = ring.axis().direction().components();
            let box_of_it = at.enclosure();
            let step = [0, 1, 2].map(|axis| box_of_it[axis] - Interval::exactly(centre[axis]));
            let squared = (0..3)
                .map(|axis| Interval::exactly(along[axis]) * Interval::exactly(along[axis]))
                .fold(Interval::exactly(0.0), |so_far, term| so_far + term);
            let shadow = (0..3)
                .map(|axis| Interval::exactly(along[axis]) * step[axis])
                .fold(Interval::exactly(0.0), |so_far, term| so_far + term);
            let out = (0..3)
                .map(|axis| step[axis] * step[axis])
                .fold(Interval::exactly(0.0), |so_far, term| so_far + term);
            let big = Interval::exactly(ring.ring_radius());
            let small = Interval::exactly(ring.tube_radius());
            let bulge = out - big * big - small * small;
            PredicatePoint::Constructed([0, 1, 2].map(|axis| {
                squared * bulge * step[axis]
                    + Interval::exactly(2.0) * big * big * shadow * Interval::exactly(along[axis])
            }))
        }
        other => panic!("a surface whose gradient nothing here knows: {other:?}"),
    }
}

/// How fast a surface's gradient turns along a direction: `d·H·d` for the very
/// polynomial [`gradient_at`] differentiates, with no normalization in it.
///
/// A plane's gradient is the same everywhere, so nothing turns. A ball's is
/// `x − c`, so `H` is the identity. A pipe's and a cone's are linear in the
/// point, so `H` is the constant `|w|²I − k·w wᵀ` with `k` one for a pipe and
/// `1 + slope²` for a cone. Only the ring depends on where it is asked:
/// differentiating `|w|²(x·x − R² − r²)x + 2R²(x·w)w` gives
/// `|w|²(2x xᵀ + (x·x − R² − r²)I) + 2R² w wᵀ`.
fn bend_along(surface: &Surface, at: PredicatePoint, along: [Interval; 3]) -> Interval {
    let squared_length = |vector: [f64; 3]| {
        (0..3)
            .map(|axis| Interval::exactly(vector[axis]) * Interval::exactly(vector[axis]))
            .fold(Interval::exactly(0.0), |so_far, term| so_far + term)
    };
    let shadow = |vector: [f64; 3]| {
        (0..3)
            .map(|axis| Interval::exactly(vector[axis]) * along[axis])
            .fold(Interval::exactly(0.0), |so_far, term| so_far + term)
    };
    let step_from = |origin: [f64; 3]| {
        let box_of_it = at.enclosure();
        [0, 1, 2].map(|axis| box_of_it[axis] - Interval::exactly(origin[axis]))
    };
    let along_squared = dot_of(along, along);
    match surface {
        Surface::Plane(_) => Interval::exactly(0.0),
        Surface::Sphere(_) => along_squared,
        Surface::Cylinder(pipe) => {
            let axis = pipe.axis().direction().components();
            let leaning = shadow(axis);
            squared_length(axis) * along_squared - leaning * leaning
        }
        Surface::Cone(cone) => {
            let axis = cone.axis().direction().components();
            let leaning = shadow(axis);
            let slope = Interval::exactly(cone.slope());
            let steepness = Interval::exactly(1.0) + slope * slope;
            squared_length(axis) * along_squared - steepness * leaning * leaning
        }
        Surface::Torus(ring) => {
            let axis = ring.axis().direction().components();
            let step = step_from(ring.centre().coordinates());
            let out = dot_of(step, step);
            let big = Interval::exactly(ring.ring_radius());
            let small = Interval::exactly(ring.tube_radius());
            let bulge = out - big * big - small * small;
            let reach = dot_of(step, along);
            let leaning = shadow(axis);
            squared_length(axis) * (Interval::exactly(2.0) * reach * reach + bulge * along_squared)
                + Interval::exactly(2.0) * big * big * leaning * leaning
        }
        other => panic!("a surface whose gradient nothing here knows: {other:?}"),
    }
}

fn dot_of(one: [Interval; 3], other: [Interval; 3]) -> Interval {
    (0..3)
        .map(|axis| one[axis] * other[axis])
        .fold(Interval::exactly(0.0), |so_far, term| so_far + term)
}

/// How points along `surface ∩ partner` are put in order, read at a point `at`
/// that is already known to be on both.
///
/// # Errors
///
/// [`Error::ACrossingThisCannotBeOrderedAlong`] for a pair whose crossing is
/// none of an endless straight line, one closed curve lying in a plane, or a
/// plane's endless bend across a cone — a ball against a pipe, or two pipes.
/// That is a **gap**: the curve is there and ordering points along it takes the
/// marcher rather than a determinant (§34.10).
///
/// [`Error::Undecidable`](crate::Error::Undecidable) when the shape of the
/// crossing itself cannot be told — two pipes whose axes cannot be shown to run
/// along one line or shown not to, and a plane and a cone whose apex or whose
/// lean cannot be placed, both of which are
/// [`a_pair_that_crosses_in_straight_lines`]'s own refusal — or when the
/// direction of a run that *is* straight comes out as a box straddling every
/// axis, or when nothing can say which way round a bend's own direction runs.
fn how_a_crossing_runs(
    surface: &Surface,
    partner: &Surface,
    run: &NamedRun,
    at: PredicatePoint,
    log: &mut ProvenanceLog,
) -> Result<HowItRuns> {
    if let (Some(one), Some(other)) = (surface.as_plane(), partner.as_plane()) {
        let (axis, forward_climbs_the_axis) = dominant_axis(one, other, log)?;
        return Ok(HowItRuns::AlongAnAxis {
            axis,
            forward_climbs_the_axis,
        });
    }
    // **A plane resting *on* a pipe hands the zero vector to the line below, and
    // that is an open gap.** A wall on `y = -2` against a bar of radius two is
    // tangent along the ruling `(0, -2, z)`, where the plane's normal `(0,-1,0)`
    // and the pipe's gradient `(0,-2,0)` are parallel, so `∇S × ∇Q` — the only
    // form this kernel has for the *direction* of a crossing, here, in
    // `dominant_axis` and in `Arrangement::way_out_of` alike — is exactly
    // nothing.
    // The direction *exists*: it is the pipe's own axis. Four families of fix,
    // and why none of them is a small change:
    //
    // 1. Read the pipe's axis here instead of the cross product. Closes this
    //    line and nothing else: `Arrangement::way_out_of` is the same cross
    //    product and is still nothing, so the wall face has no directions to
    //    sort.
    // 2. Recover the sign by hand. There is none to recover. The two rulings of
    //    a plane that really cuts a pipe get *opposite* orientations from this
    //    very cross product, so a rule handing back the axis uniformly would
    //    turn one of them round and change answers that are right today.
    // 3. Store one direction on `Crossing` and read it everywhere. This is the
    //    one that works, and it is necessary but **not sufficient**:
    //    `Arrangement::turns_anticlockwise` is `det(∇S, ∇Q₁, ∇Q₂)`, which is
    //    identically nothing against *every* other crossing once `∇Q₁` is along
    //    `∇S`, and `Arrangement::which_way_two_tangent_curves_bend` dies with it
    //    because its `B` works out to `−c|∇S × ∇Q|²`. Both derivatives taken
    //    here are blind there, so the turn predicate needs a second form too.
    // 4. Leave the crossing undrawn — it touches the wall and cuts nothing off
    //    it. `cuts.rs`'s own module comment turns down the same shape of skip
    //    for the same reason: leave a pair out and only one side of a shared
    //    edge gets split, and the answer is not a manifold (§17.4).
    if a_pair_that_crosses_in_straight_lines(surface, partner, log)? {
        let (axis, forward_climbs_the_axis) = dominant_axis_of_a_direction(
            cross_of(gradient_at(surface, at), gradient_at(partner, at)),
            log,
        )?;
        return Ok(HowItRuns::AlongAnAxis {
            axis,
            forward_climbs_the_axis,
        });
    }
    // A plane against a cone is the one pair here whose crossing can run on for
    // ever *and* bend, and which of the two it is takes no new arithmetic: it is
    // the same lean `intersect` splits its own table on. Leaning steeper than
    // the cone's own side cuts an ellipse and falls through to the conic gate;
    // level with the side cuts a parabola and shallower cuts a hyperbola, and
    // both of those are endless bends.
    if let Some((flat, cone)) = a_plane_and_a_cone(surface, partner) {
        match how_a_plane_leans_against_a_cone(flat, cone) {
            Decision::Pos => {}
            Decision::Zero | Decision::Neg => {
                return how_an_endless_bend_runs(surface, partner, flat, cone, at, log);
            }
            Decision::Undecidable { .. } | _ => {
                return Err(gave_up(
                    A_PLANE_AGAINST_A_CONE,
                    &[
                        PredicatePoint::from(flat.normal().components()),
                        PredicatePoint::from(cone.axis().direction().components()),
                    ],
                    Measures::Length,
                    log,
                ));
            }
        }
    }
    // Everything left is either a conic or a quartic, and only the conic can be
    // ordered: three points on a convex curve run round it in the order they
    // wind, which is one determinant (§34.5). A quartic closes too — a ball
    // crossing a pipe leaves loops — so *finding a place to start* on one says
    // nothing about being able to order it, and reading the two as the same
    // question is what gave a ball with a pipe through it four pieces where it
    // has three.
    if !a_pair_that_crosses_in_a_conic(surface, partner) {
        return Err(Error::ACrossingThisCannotBeOrderedAlong);
    }
    match where_a_closed_crossing_starts(run, [surface, partner], log) {
        Ok(from) => Ok(HowItRuns::RoundAClosedCurve {
            from: PredicatePoint::from(from.coordinates()),
        }),
        Err(_) => Err(Error::ACrossingThisCannotBeOrderedAlong),
    }
}

/// The pair as a plane and a cone, whichever way round it arrived.
///
/// Two `if let`s rather than one `match`, because a match over two
/// `#[non_exhaustive]` surfaces needs a wildcard arm and there is nothing here
/// for a wildcard to say (§40).
fn a_plane_and_a_cone<'s>(one: &'s Surface, other: &'s Surface) -> Option<(&'s Plane, &'s Cone)> {
    if let (Some(flat), Surface::Cone(cone)) = (one.as_plane(), other) {
        return Some((flat, cone));
    }
    if let (Surface::Cone(cone), Some(flat)) = (one, other.as_plane()) {
        return Some((flat, cone));
    }
    None
}

/// Which way round an endless bend's own direction runs, read once at a point
/// already known to be on both surfaces.
///
/// The order along the bend is how far along `n × w` a point sits
/// ([`along_a_bend`] is the derivation), and `n × w` is a direction of the
/// plane's choosing that knows nothing about which way the arrangement walks
/// this crossing. Everything else here walks it along `∇S × ∇Q`, so the two
/// have to be brought together, and this is the one sign that does it — the
/// same job [`dominant_axis_of_a_direction`] does for a straight run, where
/// `forward_climbs_the_axis` comes out of the very same cross product.
///
/// **One reading holds for the whole curve.** `∇S × ∇Q` is the tangent, and a
/// tangent's component across the bend's axis never changes sign: a parabola
/// `vertex + across·s + bend·s²` has tangent `across + 2·bend·s`, whose part
/// along `across` is `‖across‖²`, and a hyperbola branch
/// `centre + out·cosh σ + across·sinh σ` has tangent `out·sinh σ + across·cosh σ`,
/// whose part along `across` is `‖across‖² cosh σ` — constant and positive on
/// one, positive on the other. So the sign this reads at one point is the sign
/// everywhere on the run.
///
/// # Errors
///
/// [`Error::Undecidable`](crate::Error::Undecidable) when the sign cannot be
/// proved either way. [`Decision::Zero`] is refused with it and is not a
/// separate answer: `n × w` is nothing only where the plane is square to the
/// cone's axis, which cuts a circle and never reaches here, and the tangent is
/// nothing only where the plane is tangent to the cone, which needs the apex on
/// the plane and is a pair of rulings. So a `Zero` here is arithmetic that ran
/// out and not a shape, and the honest thing to do with it is give up.
fn how_an_endless_bend_runs(
    surface: &Surface,
    partner: &Surface,
    flat: &Plane,
    cone: &Cone,
    at: PredicatePoint,
    log: &mut ProvenanceLog,
) -> Result<HowItRuns> {
    let plane_normal = flat.normal().components();
    let cone_axis = cone.axis().direction().components();
    let forward = cross_of(gradient_at(surface, at), gradient_at(partner, at));
    let with_the_bend = dot_of(forward, across_a_bend(plane_normal, cone_axis));
    let forward_runs_with_the_bend = match with_the_bend.sign() {
        Decision::Pos => true,
        Decision::Neg => false,
        Decision::Zero | Decision::Undecidable { .. } | _ => {
            return Err(gave_up(
                WHICH_WAY_A_CROSSING_RUNS,
                &[
                    PredicatePoint::Constructed(forward),
                    PredicatePoint::from(plane_normal),
                    PredicatePoint::from(cone_axis),
                ],
                Measures::Volume,
                log,
            ));
        }
    };
    Ok(HowItRuns::AlongABend {
        plane_normal,
        cone_axis,
        forward_runs_with_the_bend,
    })
}

/// The direction points along an endless bend are put in order by: `n × w`, for
/// the plane's normal and the cone's axis direction.
///
/// The two are stored triples, so every product here is one multiplication of
/// two `f64`s and the range round it is as tight as an interval gets.
fn across_a_bend(plane_normal: [f64; 3], cone_axis: [f64; 3]) -> [Interval; 3] {
    cross_of(
        PredicatePoint::from(plane_normal),
        PredicatePoint::from(cone_axis),
    )
}

/// Which of two points comes first along an endless bend: [`Decision::Pos`]
/// when `first` does, [`Decision::Zero`] when they are the same point.
///
/// # Why a comparison across the bend's axis orders the whole curve
///
/// The bend lies in the plane, and the plane's copy of the cone's axis —
/// `(n·n)w − (n·w)n`, the axis projected into the plane — is the bend's own
/// axis of symmetry: reflecting in the plane spanned by `n` and `w` carries the
/// cone to itself and the plane to itself, so it carries the crossing to itself.
/// Square to that axis, inside the plane, is
/// `n × ((n·n)w − (n·w)n) = (n·n)(n × w)`, and `n·n` is positive, so **`n × w`
/// is the across-direction up to a positive factor**.
///
/// Both endless conics run one way along it. A parabola is
/// `vertex + across·s + bend·s²` with `across` square to `bend`, so its part
/// along `across` is `‖across‖²s`; one branch of a hyperbola is
/// `centre + out·cosh σ + across·sinh σ` with `across` square to `out`, so its
/// part along `across` is `‖across‖² sinh σ`. Both are strictly increasing in
/// the parameter, so a point's place along `n × w` is a **total** order along
/// the whole run and not merely a local one.
///
/// That is also why [`Decision::Zero`] means one point here. Two points of the
/// run differing by nothing along `n × w` differ by a vector that is in the
/// plane and square to `n × w`, which is a multiple of the axis; the two
/// parametrisations above hold no such pair, because equal `s` and equal
/// `sinh σ` each mean equal parameter.
///
/// # It is a determinant and never a built vector
///
/// `(second − first)·(n × w)` is `det(second − first, n, w)`, and it is asked
/// as one — [`orient3d_of_points`] against the origin, with the cascade behind
/// it — rather than by building `n × w` as three `f64`s and dotting. §48.10
/// measured what the difference is worth on the neighbouring triple product: a
/// built vector left `4.65e-17` where the determinant leaves nothing, and
/// called the passing result luck rather than cancellation.
fn along_a_bend(
    plane_normal: [f64; 3],
    cone_axis: [f64; 3],
    forward_runs_with_the_bend: bool,
    first: PredicatePoint,
    second: PredicatePoint,
) -> Decision {
    let (mine, theirs) = (second.enclosure(), first.enclosure());
    let apart = PredicatePoint::Constructed([0, 1, 2].map(|axis| mine[axis] - theirs[axis]));
    let decided = orient3d_of_points(
        as_exact_as_it_is(apart),
        PredicatePoint::from(plane_normal),
        PredicatePoint::from(cone_axis),
        PredicatePoint::from([0.0; 3]),
    );
    if forward_runs_with_the_bend {
        decided
    } else {
        the_other_way(decided)
    }
}

/// Whether the pair crosses in one closed curve lying in a plane.
///
/// The same arms [`which_comes_first_round_a_crossing`] can answer about, asked
/// before anything is built rather than after (§34.10's last two rows).
///
/// # A plane and a torus is a row, and it does not read the placement
///
/// Only two families of plane cut a torus in circles — square to the axis, and
/// through it — and every other plane cuts it in a quartic. That is a fact about
/// where the plane sits and not about the two kinds, so this cannot decide it.
/// It does not have to: [`intersect`](rcad_geom::intersect) names a plane-and-
/// torus crossing **only** in those two families and refuses the rest by name,
/// so a run of that pair reaching here is a circle, and the pair is asked the
/// same three signs again by [`which_comes_first_round_a_crossing`] when a
/// point is actually ordered. Where the two disagree — `intersect` asks under a
/// policy and the ordering asks exactly — the ordering **refuses**, and a
/// refusal is what this function returning `true` too often costs. Returning it
/// too *rarely* costs a bend every fuse it has, which is what it used to.
///
/// # All twenty-five ordered pairs are named, and that is not the guard
///
/// This was seven `|`-ed pairs out of twenty-five under a `matches!`, and a cone
/// and a torus landed this milestone and fell out of it in silence — the same
/// shape as `walk`'s three hand-kept lists (§24.5), and the fifth time this
/// milestone that a list of surface pairs did not learn about a new surface
/// kind. Naming all twenty-five puts every pair in front of a reader and leaves
/// the `_` covering only a kind that does not exist, which is §40's form.
///
/// **It does not make a sixth [`Surface`] a compile error, and nothing in this
/// crate can.** [`Surface`] is `#[non_exhaustive]` and lives in `rcad-geom`, so
/// a match over it here must carry a wildcard; a sixth kind adding zero errors
/// to `cargo check -p rcad-topo` is measured in
/// `tests/a_sixth_kind_of_surface.rs`, which is the guard that does fire.
fn a_pair_that_crosses_in_a_conic(one: &Surface, other: &Surface) -> bool {
    match (one, other) {
        // A plane cuts a ball, a pipe and a cone in a conic lying in the plane.
        // The two that cut straight lines instead — a plane along a pipe's axis,
        // and a plane through a cone's apex leaning no shallower than the cone —
        // never reach here: `a_pair_that_crosses_in_straight_lines` answers both
        // one step up. Neither does a plane whose bend across a cone never
        // closes, which `how_a_plane_leans_against_a_cone` picks off one step up
        // as well, so a plane and a cone arriving here is an ellipse.
        (Surface::Plane(_), Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_))
        | (Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_), Surface::Plane(_))
        // Two balls cross in the circle their radical plane cuts.
        | (Surface::Sphere(_), Surface::Sphere(_))
        // A plane and a torus: circles in the two families `intersect` names,
        // and no run of any other family exists to be asked.
        | (Surface::Plane(_), Surface::Torus(_))
        | (Surface::Torus(_), Surface::Plane(_)) => true,
        // Two planes cross in a straight line and are answered one step up, so
        // this arm is about the shape of the crossing and not about reaching it.
        (Surface::Plane(_), Surface::Plane(_))
        // A ball against a pipe, a cone or a ring crosses in a quartic.
        | (Surface::Sphere(_), Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_))
        | (Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_), Surface::Sphere(_))
        // So does every pair among the pipe, the cone and the ring, bar two
        // side-by-side pipes, which cut straight lines. `false` is right for
        // them here — a pair of endless lines is not a closed conic — and they
        // never reach here anyway: `a_pair_that_crosses_in_straight_lines`
        // answers them one step up.
        | (
            Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_),
            Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_),
        ) => false,
        _ => false,
    }
}

/// Whether the pair crosses in endless straight lines rather than in a closed
/// curve: a plane parallel to a pipe's axis, a plane through a cone's apex that
/// is no shallower than the cone's own side, or two pipes whose axes run along
/// one direction.
///
/// Two planes cross in a straight line too, and are answered a step up before
/// this is asked.
///
/// **A plane against a pipe is [`a_plane_lies_along_a_pipes_axis`], and it used
/// to be `flat.normal().dot(pipe.axis().direction()) == 0.0`.** That is the same
/// trap the pipe-and-pipe row below was in: a sum of three products rounding to
/// nothing is not a proof that it is nothing, and a plane it wrongly calls flush
/// with the axis has its closed ellipse ordered along an axis instead.
/// `a_plane_a_rounded_dot_calls_flush_with_an_axis_is_not` is the
/// demonstration.
///
/// **A plane against a cone is [`a_plane_cuts_a_cone_in_rulings`], and it was in
/// the same trap twice over** — an `f64` distance to the apex and an `f64` `E`,
/// both rounded, both able to go the wrong way. Its own doc carries the pair of
/// numbers and what each way of being wrong costs;
/// `a_rounded_apex_and_a_rounded_lean_together_call_a_closed_ellipse_two_rulings`
/// is the demonstration.
///
/// The other pairs are named rather than swept up, because `false` for them is a
/// fact about the question and not about the shapes: nothing but a plane holds a
/// straight line with a ball or a ring on the other side of it, and a cone
/// against a ball, a pipe or another cone is a degree-four space curve.
///
/// **Two pipes side by side are the third straight row, and both doors ask
/// [`two_pipes_run_along_one_line`] for it.** `rcad-geom`'s
/// `where_two_pipes_side_by_side_cross` builds one line or two for exactly that
/// pair and its own doc says *"every answer is straight"*. This used to answer
/// `false`, so such a run fell past this gate into the conic gate — `false`
/// there too, because a pair of endless lines does not close — and
/// [`how_a_crossing_runs`] refused it with nothing having run out of
/// arithmetic. That was a **gap** and not a limit (§16.10), and
/// `two_pipes_side_by_side_run_up_their_common_axis` measures what it answers
/// now rather than what it used to refuse.
///
/// **Two pipes exactly touching along one ruling are still refused, and that is
/// the same limit as a plane resting on a pipe.** The axes are parallel, so this
/// says `true` and the line under the gate is `∇S × ∇Q` — which for two pipes
/// touching along a ruling is exactly nothing, both gradients pointing along the
/// line between the axes. The refusal moves from
/// [`Error::ACrossingThisCannotBeOrderedAlong`] to [`Error::Undecidable`], which
/// is [`how_a_crossing_runs`]'s own note on the four families of fix and why the
/// working one is not a small change.
/// `two_pipes_resting_against_each_other_have_no_direction_to_read` pins it.
///
/// The `_`
/// at the bottom is for a sixth kind and nothing else (§40) — but a sixth kind
/// falls into it silently rather than failing to compile, because [`Surface`] is
/// `#[non_exhaustive]` and belongs to another crate.
/// `tests/a_sixth_kind_of_surface.rs` is what notices, and says why the compiler
/// cannot.
///
/// # Errors
///
/// [`Error::Undecidable`](crate::Error::Undecidable) when
/// [`two_pipes_run_along_one_line`] or [`a_plane_lies_along_a_pipes_axis`] can
/// prove neither that the two directions are flush nor that they are not, and
/// when [`a_plane_cuts_a_cone_in_rulings`] can prove neither of *its* two signs.
/// Neither `true` nor `false` is the safe reading of that: `true` orders a
/// closed curve as an endless straight line, and `false` sends a pair of endless
/// lines to the conic gate.
fn a_pair_that_crosses_in_straight_lines(
    one: &Surface,
    other: &Surface,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    match (one, other) {
        (Surface::Plane(flat), Surface::Cylinder(pipe))
        | (Surface::Cylinder(pipe), Surface::Plane(flat)) => {
            match a_plane_lies_along_a_pipes_axis(flat, pipe) {
                Decision::Zero => Ok(true),
                Decision::Neg | Decision::Pos => Ok(false),
                Decision::Undecidable { .. } | _ => Err(gave_up(
                    A_PLANE_AGAINST_AN_AXIS,
                    &[
                        PredicatePoint::from(flat.normal().components()),
                        PredicatePoint::from(pipe.axis().direction().components()),
                    ],
                    Measures::Length,
                    log,
                )),
            }
        }
        (Surface::Plane(flat), Surface::Cone(cone))
        | (Surface::Cone(cone), Surface::Plane(flat)) => {
            match a_plane_cuts_a_cone_in_rulings(flat, cone) {
                Some(straight) => Ok(straight),
                None => Err(gave_up(
                    A_PLANE_AGAINST_A_CONE,
                    &[
                        PredicatePoint::from(flat.normal().components()),
                        PredicatePoint::from(cone.axis().direction().components()),
                    ],
                    Measures::Length,
                    log,
                )),
            }
        }
        (Surface::Cylinder(one), Surface::Cylinder(other)) => {
            match two_pipes_run_along_one_line(one, other) {
                Decision::Zero => Ok(true),
                Decision::Pos => Ok(false),
                Decision::Neg => unreachable!("a squared length is never below zero"),
                Decision::Undecidable { .. } | _ => Err(gave_up(
                    ORIENT2D,
                    &[one, other]
                        .map(|pipe| PredicatePoint::from(pipe.axis().direction().components())),
                    Measures::Area,
                    log,
                )),
            }
        }
        (Surface::Plane(_), Surface::Plane(_) | Surface::Sphere(_) | Surface::Torus(_))
        | (Surface::Sphere(_) | Surface::Torus(_), Surface::Plane(_))
        | (Surface::Sphere(_) | Surface::Torus(_), Surface::Sphere(_) | Surface::Torus(_))
        | (Surface::Cylinder(_) | Surface::Cone(_), Surface::Sphere(_) | Surface::Torus(_))
        | (Surface::Sphere(_) | Surface::Torus(_), Surface::Cylinder(_) | Surface::Cone(_))
        | (Surface::Cylinder(_), Surface::Cone(_))
        | (Surface::Cone(_), Surface::Cylinder(_) | Surface::Cone(_)) => Ok(false),
        _ => Ok(false),
    }
}

/// Whether a plane cuts a cone in rulings rather than in a conic that closes.
///
/// The two signs [`where_a_plane_and_a_cone_cross`](rcad_geom::intersect) reads
/// to tell its six cases apart, **asked of the very functions it asks**:
/// [`where_a_cones_apex_stands_against_a_plane`] for `n·a − d` and
/// [`how_a_plane_leans_against_a_cone`] for `E = (n·w)² − slope²‖n × w‖²`, both
/// of them `rcad-geom`'s. With the apex on the plane and `E` at or below nothing
/// the crossing is one ruling the plane rests along (`E = 0`) or two rulings
/// meeting at the apex (`E < 0`), and both are endless straight lines.
///
/// Every other pair of signs is a curve: off the plane the crossing is an
/// ellipse, a parabola or a hyperbola, and through the apex with `E > 0` the two
/// share the apex alone, which is no curve at all.
///
/// **What is left here is the table and not the arithmetic.** The two
/// expressions used to be written out a second time in this file, kept honest by
/// a 360 000-draw test that asserted this door and `intersect` agreed on every
/// draw; that guard is not the same thing as not having a copy (§49.8), and the
/// copy is gone. The draws remain, and what they now check is this table —
/// whether a pair of signs means straight or curved — against the shape
/// `intersect` names for the same plane and cone.
///
/// **Both signs are proved and neither is rounded**, and [`None`] is the third
/// answer for a pair where the arithmetic runs out. `Some(true)` is rulings,
/// `Some(false)` is a curve or the apex alone, and neither is the safe reading
/// of a pair nobody has decided: `true` orders a closed ellipse along an axis
/// and `false` puts two arcs where a ruling has one.
///
/// **Neither sign goes through a [`Policy`], and that is the point of them
/// taking none.** `intersect` asks the same two questions with the caller's
/// policy behind them, so a tolerant one can read an enclosure the cascade
/// refused as nothing; this door gets the cascade and stops there. The two part
/// company only under a tolerant policy and only into a refusal — `None` here,
/// where `intersect` was handed an answer — and never into this door calling a
/// closed curve two rulings, which is the trade the whole row exists to avoid.
///
/// # This used to be rounded, and it said a closed ellipse was an endless line
///
/// It was `flat.signed_distance_to(cone.apex()) != 0.0` and then
/// `up*up - slope*slope*sideways <= 0.0`, with the doc claiming that
/// disagreeing with [`intersect`](rcad_geom::intersect) cost a refusal rather
/// than a wrong answer. **It cost a wrong answer**, the same one the two rows
/// above cost before them (§49.2), and it takes both signs being wrong at once:
///
/// - the apex a rounding off the plane, which the `f64` distance reads as zero
///   — the plane-and-pipe trap one row up, with the offset subtracted as well;
/// - the plane a rounding off tangent to the cone, which sends `E` — degree six
///   in the stored numbers — the wrong side of zero.
///
/// Both together and `intersect` names one closed **ellipse** while this says
/// two rulings, so [`how_a_crossing_runs`] orders that ellipse along an axis.
/// `a_rounded_apex_and_a_rounded_lean_together_call_a_closed_ellipse_two_rulings`
/// writes one such pair out.
///
/// # What each way of being wrong actually costs
///
/// Measured rather than argued.
/// `how_many_near_tangent_planes_a_rounded_cone_gate_gets_wrong` draws 360 000
/// planes against cones — random normals, normals a rounding off tangent, and
/// apexes both nudged a rounding off the plane and left exactly on it — puts
/// every one of them to `intersect`, and counts what the rounded pair of tests
/// would have said:
///
/// | the gate said | the truth | how many | what happened |
/// |---|---|---|---|
/// | rulings | one closed ellipse | **3 655** | **ordered along an axis**, all 3 655 of them: the wrong answer |
/// | rulings | the apex alone | 16 077 | nothing: `intersect` names no run to order |
/// | a curve | two rulings | 7 761 | nothing **today**: every one was refused by `intersect` for want of an exactly representable ruling direction. Nothing but that refusal stands between it and the two-arc branch of `which_arc_this_is`, so this sign is proved here too rather than left to it |
///
/// The proved gate disagreed with `intersect` about **none** of the 360 000.
pub(super) fn a_plane_cuts_a_cone_in_rulings(flat: &Plane, cone: &Cone) -> Option<bool> {
    match where_a_cones_apex_stands_against_a_plane(flat, cone) {
        Decision::Zero => {}
        Decision::Pos | Decision::Neg => return Some(false),
        Decision::Undecidable { .. } | _ => return None,
    }
    match how_a_plane_leans_against_a_cone(flat, cone) {
        Decision::Zero | Decision::Neg => Some(true),
        Decision::Pos => Some(false),
        Decision::Undecidable { .. } | _ => None,
    }
}

/// How many entries the exact dot product of two stored triples can need: one
/// two-entry product per axis.
const LONGEST_DOT: usize = 6;

/// The exact dot product of two stored triples, written into `out`.
///
/// [`None`] when one of the three products would round, which is what
/// [`product_is_exact`] is for — not that the answer is zero.
fn exactly_the_dot_of(one: [f64; 3], other: [f64; 3], out: &mut [f64]) -> Option<usize> {
    let mut total = [0.0; LONGEST_DOT];
    let mut len = 0;
    for axis in 0..3 {
        if !product_is_exact(one[axis], other[axis]) {
            return None;
        }
        let (high, low) = two_product(one[axis], other[axis]);
        let mut grown = [0.0; LONGEST_DOT];
        len = expansion_sum(&total[..len], &[low, high], &mut grown);
        total[..len].copy_from_slice(&grown[..len]);
    }
    out[..len].copy_from_slice(&total[..len]);
    Some(len)
}

/// Whether two pipes' axes run along one line, **proved and not rounded**.
///
/// [`Decision::Zero`] when `w₁ × w₂` is nothing, [`Decision::Pos`] when it is
/// something, and [`Decision::Undecidable`] when neither can be shown. That is
/// the sign of `‖w₁ × w₂‖²`, which is the very question
/// [`intersect`](rcad_geom::intersect) splits the pipe-and-pipe row on, so this
/// door, `assemble.rs`'s [`the_crossing_of_these_runs_straight`] and the
/// intersector all ask one question of one pair of stored directions.
///
/// **Under [`Policy::certified`] that is the same arithmetic and the same
/// answer.** Under a tolerant policy `intersect` can read an enclosure this
/// refuses as nothing, and then the two part company — but only into a
/// **refusal**: this door hands the pair to the conic gate or gives up, and
/// neither builds an order. What cannot happen either way is a crossing
/// `intersect` names as closing being ordered as an endless line, which is what
/// the rounded test allowed.
///
/// [`the_crossing_of_these_runs_straight`]: super::assemble
///
/// # An `f64` cross product rounding to nothing is not a proof
///
/// Both doors used to write `w₁.cross(w₂) == Vector::zero()`. Take the unit
/// directions `(a, 0, c)` and `(b, 0, c)` with
/// `a = 0.39350772579388144`, `b = 0.3935077257938814` and
/// `c = 0.919321309304058`: `a` and `b` differ by one ulp, `a·c` and `b·c` round
/// to the same `f64`, so the cross product's `y` component *rounds* to nothing
/// while it exactly is `(b − a)·c ≈ −5.1e-17`. Those two pipes cross in a closed
/// quartic loop, and the rounded test called the loop an endless straight line.
///
/// Each component is a two-by-two determinant of stored numbers, which is
/// [`orient2d`] measured from the origin — the same form
/// [`dominant_axis`] reads a pair of plane normals with, exact for exact `f64`
/// inputs and honest about the spread it cannot multiply through. One component
/// proved non-zero settles the pair on its own; the three have to be proved
/// *zero* together for the axes to be along one line.
pub(super) fn two_pipes_run_along_one_line(one: &Cylinder, other: &Cylinder) -> Decision {
    let (mine, theirs) = (
        one.axis().direction().components(),
        other.axis().direction().components(),
    );
    let mut widest_that_was_not_told = None::<f64>;
    for axis in 0..3 {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        match orient2d(
            [0.0, 0.0],
            [mine[next], mine[after]],
            [theirs[next], theirs[after]],
        ) {
            Decision::Zero => {}
            Decision::Pos | Decision::Neg => return Decision::Pos,
            Decision::Undecidable { enclosure } => {
                widest_that_was_not_told = Some(
                    widest_that_was_not_told.map_or(enclosure, |so_far| so_far.max(enclosure)),
                );
            }
            other => panic!("a two-by-two determinant came back with no sign: {other:?}"),
        }
    }
    match widest_that_was_not_told {
        None => Decision::Zero,
        Some(enclosure) => Decision::Undecidable { enclosure },
    }
}

/// Whether a plane lies along a pipe's axis, **proved and not rounded**.
///
/// The sign of `n·w`. [`Decision::Zero`] and the plane holds the axis's
/// direction, so it cuts the pipe in one endless ruling or two; anything else
/// and it cuts across, which is an ellipse that closes.
///
/// **The same trap as [`two_pipes_run_along_one_line`], one row down.** Both
/// doors wrote `flat.normal().dot(pipe.axis().direction()) == 0.0`, and a sum of
/// three products rounding to nothing is not a proof that it is nothing.
/// `intersect` decides this sign exactly, names such a crossing
/// `RightRoundWhereItsFacesCross`, and the rounded test had
/// [`how_a_crossing_runs`] order that closed ellipse along an axis.
/// `a_plane_a_rounded_dot_calls_flush_with_an_axis_is_not` is one such pair
/// written out, and
/// `how_many_planes_a_rounded_dot_calls_flush_with_an_axis_are_not` says how
/// common they are rather than leaving it at "possible".
///
/// Three products of stored numbers summed with no rounding thrown away, which
/// is level 3 of the cascade written out. `product_is_exact` is the guard
/// against a product that lands in the subnormals, where the exactness this
/// rests on stops holding; there is no bound to report for one that does, so the
/// enclosure is infinite and no policy can accept it.
pub(super) fn a_plane_lies_along_a_pipes_axis(flat: &Plane, pipe: &Cylinder) -> Decision {
    let mut reach = [0.0; LONGEST_DOT];
    let Some(len) = exactly_the_dot_of(
        flat.normal().components(),
        pipe.axis().direction().components(),
        &mut reach,
    ) else {
        return Decision::Undecidable {
            enclosure: f64::INFINITY,
        };
    };
    expansion_sign(&reach[..len])
}

/// The name the question of whether a plane holds a direction goes by in
/// provenance records.
const A_PLANE_AGAINST_AN_AXIS: &str = "a plane against an axis";

/// The name the pair of questions about where a cone's apex stands and how the
/// plane leans goes by in provenance records.
const A_PLANE_AGAINST_A_CONE: &str = "a plane against a cone";

/// The cross product of two boxes.
fn cross_of(one: PredicatePoint, other: PredicatePoint) -> [Interval; 3] {
    let (mine, theirs) = (one.enclosure(), other.enclosure());
    [0, 1, 2].map(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        mine[next] * theirs[last] - mine[last] * theirs[next]
    })
}

/// The name the ordering of two points along a crossing goes by in provenance
/// records.
const ALONG_A_CROSSING: &str = "order along a crossing";

/// The name the choice between "the same way" and "opposite ways" along one line
/// goes by in provenance records.
const THE_SAME_WAY_ALONG_ONE_LINE: &str = "two ways out of a node along one line";

/// The name the second-order order of two tangent curves goes by in provenance
/// records.
const HOW_TWO_TANGENT_CURVES_BEND: &str = "which way two tangent curves bend";

/// The name the search for the piece a hole sits in goes by in provenance
/// records.
const AROUND_A_HOLE: &str = "which piece of a face a hole is in";

/// Where a ray's own curve meets one arc of the drawing.
enum MetAt {
    /// Nowhere along it.
    Nowhere,
    /// At this node, strictly between the arc's two ends.
    Between(usize),
    /// At one of the arc's ends, or too close to one to say.
    AnEnd(usize),
}

/// Where a node sits in the order along one crossing.
///
/// # Panics
///
/// If the node is not on that crossing. [`Arrangement::place_the_corners`]
/// starts every corner's crossing list with the crossing its own edge runs
/// along, and the corner an edge stops at picks that crossing up from the
/// surfaces it already knows meet there, so both ends of an edge are on it.
fn at_in(order: &[usize], node: usize) -> usize {
    order
        .iter()
        .position(|&other| other == node)
        .expect("a corner is one of the nodes on the crossing its edge runs along")
}

/// Which axis the line `p ∩ q` climbs fastest along, and whether
/// `n_p × n_q` climbs it rather than descends it.
///
/// The line's direction is `n_p × n_q`, whose components are two-by-two
/// determinants of stored coefficients, so [`orient2d`] measured from the origin
/// gives each of their signs exactly. Picking the largest of them is done in
/// `f64` and decides nothing: any axis whose component is *proved* non-zero
/// orders the line's points correctly, and the largest is only the one that
/// separates them furthest.
///
/// # Errors
///
/// [`Error::Undecidable`](crate::Error::Undecidable) when no component can be shown to be non-zero, which
/// for two planes that are not parallel takes coefficients too extreme to
/// multiply.
fn dominant_axis(plane: &Plane, partner: &Plane, log: &mut ProvenanceLog) -> Result<(usize, bool)> {
    let (mine, theirs) = (plane.normal().components(), partner.normal().components());
    let mut best: Option<(usize, bool, f64)> = None;
    for axis in 0..3 {
        let (one, other) = ((axis + 1) % 3, (axis + 2) % 3);
        let size = (mine[one] * theirs[other] - mine[other] * theirs[one]).abs();
        let sign = orient2d(
            [0.0, 0.0],
            [mine[one], mine[other]],
            [theirs[one], theirs[other]],
        );
        let climbs = match sign {
            Decision::Pos => true,
            Decision::Neg => false,
            Decision::Zero | Decision::Undecidable { .. } | _ => continue,
        };
        if best.is_none_or(|(_, _, so_far)| so_far < size) {
            best = Some((axis, climbs, size));
        }
    }
    match best {
        Some((axis, climbs, _)) => Ok((axis, climbs)),
        None => Err(gave_up(
            ORIENT2D,
            &[PredicatePoint::from(mine), PredicatePoint::from(theirs)],
            Measures::Area,
            log,
        )),
    }
}

/// The same, for a direction that had to be worked out rather than read off two
/// stored normals.
///
/// The components are ranges, so an axis is only usable when its range is
/// **proved** clear of zero. Widest first, for the same reason as above: any
/// proved axis orders the points correctly and the widest separates them most.
fn dominant_axis_of_a_direction(
    direction: [Interval; 3],
    log: &mut ProvenanceLog,
) -> Result<(usize, bool)> {
    let mut best: Option<(usize, bool, f64)> = None;
    for (axis, along) in direction.iter().enumerate() {
        let climbs = match along.sign() {
            Decision::Pos => true,
            Decision::Neg => false,
            Decision::Zero | Decision::Undecidable { .. } | _ => continue,
        };
        let size = along.lo().abs().min(along.hi().abs());
        if best.is_none_or(|(_, _, so_far)| so_far < size) {
            best = Some((axis, climbs, size));
        }
    }
    match best {
        Some((axis, climbs, _)) => Ok((axis, climbs)),
        None => Err(gave_up(
            WHICH_WAY_A_CROSSING_RUNS,
            &[PredicatePoint::Constructed(direction)],
            Measures::Area,
            log,
        )),
    }
}

/// The name the direction of a crossing that had to be worked out goes by in
/// provenance records.
///
/// Deliberately not [`ORIENT2D`], which is what the two-plane case logs. That
/// one reads two stored normals and is exact; this one reads a *gradient at a
/// box* and gives up where the box is wide enough to straddle an axis, so a
/// breakdown that called them one thing would hide which of the two ran out.
const WHICH_WAY_A_CROSSING_RUNS: &str = "which way a crossing runs";

/// Whether two planes are the same plane, refusing when nothing can tell.
///
/// [`PlaneAgreement::CannotTell`] takes coefficients whose products fall out of
/// the range `f64` multiplication can be undone in — §3.5's dynamic-range check
/// is what keeps models like that out. Nothing can bound how wrong the answer
/// would be, so the record says so rather than naming a width it does not have.
pub(crate) fn agreement_of_two_planes(
    plane: &Plane,
    other: &Plane,
    log: &mut ProvenanceLog,
) -> Result<PlaneAgreement> {
    match plane.agreement_with(other) {
        settled @ (PlaneAgreement::SameWay
        | PlaneAgreement::OppositeWay
        | PlaneAgreement::Different) => Ok(settled),
        PlaneAgreement::CannotTell => Err(gave_up(
            ORIENT2D,
            &[
                PredicatePoint::from(plane.normal().components()),
                PredicatePoint::from(other.normal().components()),
            ],
            Measures::Area,
            log,
        )),
        other => panic!("plane agreement came back with an answer nothing here knows: {other:?}"),
    }
}

/// How the lines `p ∩ first` and `p ∩ second` lie.
enum LineRelation {
    /// They cross at one point.
    CrossSomewhere,
    /// They run alongside each other and never meet.
    Parallel,
    /// They are the same line.
    Same,
}

/// Which of the three it is, decided exactly.
///
/// The three planes share a line exactly when the three-by-four matrix of their
/// coefficients has rank two, and that is when all four of its three-by-three
/// minors vanish. Each minor is the determinant of three `f64` triples, which is
/// [`orient3d`] measured from the origin — so this is four exact determinants of
/// stored numbers, with no constructed point and nothing to be undecided about
/// short of an overflow.
///
/// The first minor is the one built from the normals. Non-zero, and the three
/// planes meet at a point, so the two lines cross. Zero, and the lines are
/// parallel or the same, and the other three minors say which.
fn relation_of_two_lines(
    plane: &Plane,
    first: &Plane,
    second: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<LineRelation> {
    let rows = [
        plane.coefficients(),
        first.coefficients(),
        second.coefficients(),
    ];
    let minor = |columns: [usize; 3], log: &mut ProvenanceLog| -> Result<Decision> {
        let pick = |row: [f64; 4]| [row[columns[0]], row[columns[1]], row[columns[2]]];
        let taken = rows.map(pick);
        must_decide(
            orient3d(taken[0], taken[1], taken[2], [0.0; 3]),
            ORIENT3D,
            &taken.map(PredicatePoint::from),
            Measures::Volume,
            policy,
            log,
        )
    };

    if minor([0, 1, 2], log)? != Decision::Zero {
        return Ok(LineRelation::CrossSomewhere);
    }
    for columns in [[0, 1, 3], [0, 2, 3], [1, 2, 3]] {
        if minor(columns, log)? != Decision::Zero {
            return Ok(LineRelation::Parallel);
        }
    }
    Ok(LineRelation::Same)
}
#[cfg(test)]
mod tests {
    use rcad_geom::{Isometry, Plane, Point, Scale, Surface, Vector};
    use rcad_predicates::{Decision, Policy, ProvenanceLog, orient3d};

    use super::{Piece, split_a_face, the_other_way, where_the_crossing_starts};
    use crate::fuse::bake::{BakedFace, bake};
    use crate::fuse::{FaceMeet, MeetPoint};
    use crate::{Model, ProfileBuilder, SolidKey};

    /// The identity the anticlockwise order of two lines round a corner rests
    /// on, checked against a determinant worked out by hand.
    ///
    /// `orient3d(a, b, c, d)` is the determinant of the rows `a-d`, `b-d`,
    /// `c-d`, so with the origin last it is the determinant of the three rows
    /// themselves. Small integers keep every product exact, so the plain `f64`
    /// determinant is a fair oracle. The brief this was built from had the
    /// origin *first*, which is the same determinant with its sign flipped —
    /// that is what a test is for.
    #[test]
    fn orient3d_with_the_origin_last_is_the_determinant_of_the_three_rows() {
        for first in -2..=2 {
            for second in -2..=2 {
                for third in -2..=2 {
                    let rows = [
                        [1.0, f64::from(first), 0.5],
                        [f64::from(second), 2.0, -1.0],
                        [0.25, f64::from(third), 3.0],
                    ];
                    let by_hand = rows[0][0] * (rows[1][1] * rows[2][2] - rows[1][2] * rows[2][1])
                        - rows[0][1] * (rows[1][0] * rows[2][2] - rows[1][2] * rows[2][0])
                        + rows[0][2] * (rows[1][0] * rows[2][1] - rows[1][1] * rows[2][0]);
                    let expected = if by_hand > 0.0 {
                        Decision::Pos
                    } else if by_hand < 0.0 {
                        Decision::Neg
                    } else {
                        Decision::Zero
                    };
                    assert_eq!(orient3d(rows[0], rows[1], rows[2], [0.0; 3]), expected);
                    assert_eq!(
                        orient3d([0.0; 3], rows[0], rows[1], rows[2]),
                        the_other_way(expected),
                        "with the origin first the sign is the other way round"
                    );
                }
            }
        }
    }

    /// Whether a baked face is the flat one pointing up the z axis.
    fn facing_up(face: &BakedFace) -> bool {
        face.surface
            .as_plane()
            .is_some_and(|flat| flat.normal() == Vector::new(0.0, 0.0, 1.0))
    }

    fn a_cube_of_side_two() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        (model, cube)
    }

    /// The top of the cube: the square from `(0, 0, 2)` to `(2, 2, 2)`, facing
    /// up.
    fn the_top_of_a_cube(model: &Model, cube: SolidKey) -> BakedFace {
        let baked = bake(
            model,
            &[cube],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a cube bakes");
        baked
            .into_iter()
            .next()
            .expect("one solid was baked")
            .faces
            .into_iter()
            .find(facing_up)
            .expect("a cube has a face pointing up")
    }

    /// The upward-facing top of a profile extruded by one.
    fn the_top_of_an_extruded(outline: &[[f64; 2]]) -> BakedFace {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let profile = ProfileBuilder::new(outline.to_vec())
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a profile");
        let solid = model
            .add_extrusion(&profile, 1.0, &Isometry::identity())
            .expect("an extrusion");
        bake(
            &model,
            &[solid],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("it bakes")
        .swap_remove(0)
        .faces
        .into_iter()
        .find(facing_up)
        .expect("an extrusion has a top")
    }

    fn cut_by(face: &BakedFace, planes: &[[f64; 4]]) -> BakedFace {
        let mut cut = face.clone();
        cut.cuts = planes
            .iter()
            .map(|&coefficients| {
                Surface::Plane(Plane::from_coefficients(coefficients).expect("a plane"))
            })
            .collect();
        cut
    }

    fn split(face: &BakedFace) -> Vec<Piece> {
        let mut log = ProvenanceLog::new();
        let pieces = split_a_face(face, &Policy::certified(), &mut log)
            .expect("a lattice-aligned face splits with nothing left undecided");
        assert!(
            log.is_empty(),
            "every decision here is exactly representable, so none needed a policy"
        );
        pieces
    }

    /// Twice the area a wire sweeps round `normal`, summed as `Σ (p × q) · n`.
    ///
    /// Exact, and that is the whole point of measuring it this way here: every
    /// corner of a lattice-aligned face is a small `f64` with a handful of
    /// significant bits, so each product and each sum is exact in `f64` and the
    /// areas of the pieces add up to the parent's area with no tolerance in
    /// sight. The signs come out right for holes as well, because a hole is
    /// wound the other way and its area is subtracted.
    fn twice_the_area(wire: &[FaceMeet], normal: [f64; 3]) -> f64 {
        wire.iter()
            .map(|side| {
                let at = |end: &MeetPoint| {
                    end.point()
                        .exact_coordinates()
                        .expect("a lattice-aligned corner is an f64")
                };
                let (p, q) = (at(&side.ends[0]), at(&side.ends[1]));
                let cross = [
                    p[1] * q[2] - p[2] * q[1],
                    p[2] * q[0] - p[0] * q[2],
                    p[0] * q[1] - p[1] * q[0],
                ];
                cross[0] * normal[0] + cross[1] * normal[1] + cross[2] * normal[2]
            })
            .sum()
    }

    fn twice_the_area_of(piece: &Piece) -> f64 {
        piece
            .wires
            .iter()
            .map(|wire| twice_the_area(wire, piece.plane().normal().components()))
            .sum()
    }

    fn every_wire_closes(piece: &Piece) {
        for wire in &piece.wires {
            for step in 0..wire.len() {
                assert_eq!(
                    wire[step].ends[1].point(),
                    wire[(step + 1) % wire.len()].ends[0].point(),
                    "a side of a piece has to start where the one before it ended"
                );
            }
        }
    }

    #[test]
    fn a_face_with_nothing_cutting_it_comes_back_as_itself() {
        let (model, cube) = a_cube_of_side_two();
        let top = the_top_of_a_cube(&model, cube);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].ancestors, top.keys);
        assert_eq!(pieces[0].surface, top.surface);
        assert_eq!(pieces[0].wires.len(), 1);
        assert_eq!(pieces[0].wires[0].len(), 4);
        assert_eq!(twice_the_area_of(&pieces[0]), 8.0, "a square of side two");
        every_wire_closes(&pieces[0]);
        for side in &pieces[0].wires[0] {
            assert!(
                matches!(side.ends[0], MeetPoint::Existing { .. }),
                "an uncut face keeps the corners the input had"
            );
            assert_eq!(side.approximation, None, "planes meet in straight lines");
        }
    }

    #[test]
    fn a_face_cut_by_one_line_through_its_middle_comes_back_as_two_pieces() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(&the_top_of_a_cube(&model, cube), &[[1.0, 0.0, 0.0, -1.0]]);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 2);
        for piece in &pieces {
            assert_eq!(piece.wires.len(), 1);
            assert_eq!(piece.wires[0].len(), 4);
            assert_eq!(twice_the_area_of(piece), 4.0, "half the square each");
            every_wire_closes(piece);
        }
    }

    #[test]
    fn a_face_cut_by_two_crossing_lines_comes_back_as_four_pieces() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(
            &the_top_of_a_cube(&model, cube),
            &[[1.0, 0.0, 0.0, -1.0], [0.0, 1.0, 0.0, -1.0]],
        );

        let pieces = split(&top);

        assert_eq!(pieces.len(), 4);
        for piece in &pieces {
            assert_eq!(piece.wires[0].len(), 4);
            assert_eq!(
                twice_the_area_of(piece),
                2.0,
                "a quarter of the square each"
            );
            every_wire_closes(piece);
        }
    }

    /// The splitter's oracle, and it needs no tolerance: the pieces of a face
    /// are a partition of it, so their areas add up to its area exactly.
    #[test]
    fn the_pieces_of_a_face_add_up_to_the_area_it_started_with() {
        let (model, cube) = a_cube_of_side_two();
        let uncut = the_top_of_a_cube(&model, cube);
        let whole = twice_the_area_of(&split(&uncut)[0]);

        for cuts in [
            vec![[1.0, 0.0, 0.0, -1.0]],
            vec![[1.0, 0.0, 0.0, -0.5], [0.0, 1.0, 0.0, -1.5]],
            vec![
                [1.0, 0.0, 0.0, -0.5],
                [0.0, 1.0, 0.0, -1.5],
                [1.0, 1.0, 0.0, -2.0],
            ],
        ] {
            let pieces = split(&cut_by(&uncut, &cuts));
            let summed: f64 = pieces.iter().map(twice_the_area_of).sum();
            assert_eq!(summed, whole, "{} pieces from {cuts:?}", pieces.len());
        }
    }

    /// The other half of the oracle. Every side of every piece is walked once,
    /// and a side that two pieces share is walked once each way — which is what
    /// makes the pieces fit together with nothing over and nothing missing.
    #[test]
    fn every_side_of_a_piece_is_walked_once_and_a_shared_one_once_each_way() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(
            &the_top_of_a_cube(&model, cube),
            &[[1.0, 0.0, 0.0, -1.0], [0.0, 1.0, 0.0, -1.0]],
        );
        let rim: Vec<Surface> = top.wires.iter().flatten().map(|edge| edge.across).collect();

        let mut walked: Vec<([f64; 3], [f64; 3], Surface)> = Vec::new();
        for piece in &split(&top) {
            every_wire_closes(piece);
            for wire in &piece.wires {
                for side in wire {
                    let at = |end: &MeetPoint| {
                        end.point()
                            .exact_coordinates()
                            .expect("a lattice-aligned corner is an f64")
                    };
                    walked.push((at(&side.ends[0]), at(&side.ends[1]), side.carried_by[1]));
                }
            }
        }

        for (position, side) in walked.iter().enumerate() {
            assert!(
                !walked[position + 1..].contains(side),
                "one side of one piece is walked twice the same way"
            );
            let back = (side.1, side.0, side.2);
            assert!(
                walked.contains(&back) || rim.contains(&side.2),
                "a side nobody walks back along has to be on the rim of the face"
            );
        }
    }

    #[test]
    fn every_piece_names_the_one_face_it_came_from() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(
            &the_top_of_a_cube(&model, cube),
            &[[1.0, 0.0, 0.0, -1.0], [0.0, 1.0, 0.0, -1.0]],
        );

        for piece in &split(&top) {
            assert_eq!(piece.ancestors, top.keys);
            assert_eq!(piece.orientation, top.orientation);
        }
    }

    /// The property §14.8's commutativity oracle needs one level down: the
    /// pieces depend on the *set* of cut lines and not on the order they were
    /// found in, so `fuse(A, B)` and `fuse(B, A)` cut the same face the same
    /// way.
    #[test]
    fn presenting_the_same_cuts_the_other_way_round_gives_the_very_same_pieces() {
        let (model, cube) = a_cube_of_side_two();
        let uncut = the_top_of_a_cube(&model, cube);
        let cuts = [
            [1.0, 0.0, 0.0, -0.5],
            [0.0, 1.0, 0.0, -1.5],
            [1.0, 1.0, 0.0, -2.0],
        ];
        let mut backwards = cuts;
        backwards.reverse();

        assert_eq!(
            split(&cut_by(&uncut, &cuts)),
            split(&cut_by(&uncut, &backwards))
        );
    }

    /// A face with a hole in it, cut by a line that misses the hole. The hole is
    /// a lump of the drawing nothing joins to the rest, so which of the two
    /// pieces it belongs to is the one thing the walk cannot work out on its
    /// own.
    #[test]
    fn a_hole_the_cut_misses_ends_up_in_the_piece_that_surrounds_it() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let plate = ProfileBuilder::new(vec![[0.0, 0.0], [8.0, 0.0], [8.0, 4.0], [0.0, 4.0]])
            .with_hole(vec![[5.0, 1.0], [7.0, 1.0], [7.0, 3.0], [5.0, 3.0]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a plate with a square hole");
        let solid = model
            .add_extrusion(&plate, 1.0, &Isometry::identity())
            .expect("an extrusion");
        let top = bake(
            &model,
            &[solid],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the plate bakes")
        .swap_remove(0)
        .faces
        .into_iter()
        .find(facing_up)
        .expect("the plate has a top");
        let cut = cut_by(&top, &[[1.0, 0.0, 0.0, -2.0]]);

        let pieces = split(&cut);

        assert_eq!(pieces.len(), 2);
        let holed = pieces
            .iter()
            .find(|piece| piece.wires.len() == 2)
            .expect("the piece the hole is in keeps the hole");
        assert_eq!(holed.wires[1].len(), 4, "the hole is still a square");
        assert_eq!(
            twice_the_area_of(holed),
            2.0 * (6.0 * 4.0 - 2.0 * 2.0),
            "the piece with the hole is the hole's area short"
        );
        let plain = pieces
            .iter()
            .find(|piece| piece.wires.len() == 1)
            .expect("the other piece has no hole");
        assert_eq!(twice_the_area_of(plain), 2.0 * 2.0 * 4.0);
        for piece in &pieces {
            every_wire_closes(piece);
        }
    }

    /// A cut that runs corner to corner. Both of its ends land exactly on a
    /// corner the face already had, so the corner has to absorb the cut line
    /// rather than a new point being made a hair away from it — which is the
    /// vertex-on-face case of roadmap §7.4's catalogue, built to land on rather
    /// than approached.
    #[test]
    fn a_cut_from_one_corner_of_a_face_to_another_gives_two_pieces_and_no_new_corners() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(&the_top_of_a_cube(&model, cube), &[[1.0, 1.0, 0.0, -2.0]]);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 2);
        for piece in &pieces {
            assert_eq!(piece.wires[0].len(), 3, "each piece is a triangle");
            assert_eq!(twice_the_area_of(piece), 4.0);
            every_wire_closes(piece);
            for side in &piece.wires[0] {
                assert!(
                    matches!(side.ends[0], MeetPoint::Existing { .. }),
                    "a cut along a diagonal makes no corner the face did not have"
                );
            }
        }
    }

    /// A face that is not convex, cut by a line that also crosses the *line* of
    /// one of its own far edges at a point in the middle of the face.
    ///
    /// That crossing is a node with nothing else through it, and leaving it in
    /// would put a corner on a piece where the piece is perfectly straight.
    #[test]
    fn a_face_that_bends_back_on_itself_is_cut_into_two_with_no_corners_invented() {
        let top = the_top_of_an_extruded(&[
            [0.0, 0.0],
            [4.0, 0.0],
            [4.0, 2.0],
            [2.0, 2.0],
            [2.0, 4.0],
            [0.0, 4.0],
        ]);

        let across = split(&cut_by(&top, &[[0.0, 1.0, 0.0, -1.0]]));
        assert_eq!(across.len(), 2);
        let mut areas: Vec<f64> = across.iter().map(twice_the_area_of).collect();
        areas.sort_by(f64::total_cmp);
        assert_eq!(areas, [2.0 * 4.0, 2.0 * 8.0]);
        for piece in &across {
            every_wire_closes(piece);
            assert_eq!(piece.wires.len(), 1);
        }
        assert_eq!(
            across
                .iter()
                .find(|piece| twice_the_area_of(piece) == 8.0)
                .expect("the strip below the cut")
                .wires[0]
                .len(),
            4,
            "the strip below the cut is a rectangle, and the crossing with the far edge's line              does not put a fifth corner on it"
        );
    }

    /// A face shaped like a U, cut by a line straight across both arms. The two
    /// pieces above the cut are on the same side of it and are still two pieces,
    /// which is the case that tells a real arrangement from clipping a polygon
    /// against a half-space.
    #[test]
    fn a_cut_across_both_arms_of_a_u_leaves_three_pieces_and_not_two() {
        let top = the_top_of_an_extruded(&[
            [0.0, 0.0],
            [6.0, 0.0],
            [6.0, 4.0],
            [4.0, 4.0],
            [4.0, 2.0],
            [2.0, 2.0],
            [2.0, 4.0],
            [0.0, 4.0],
        ]);

        let pieces = split(&cut_by(&top, &[[0.0, 1.0, 0.0, -3.0]]));

        assert_eq!(pieces.len(), 3);
        let mut areas: Vec<f64> = pieces.iter().map(twice_the_area_of).collect();
        areas.sort_by(f64::total_cmp);
        assert_eq!(areas, [2.0 * 2.0, 2.0 * 2.0, 2.0 * 16.0]);
        for piece in &pieces {
            every_wire_closes(piece);
        }
    }

    /// A cut that lands exactly on an edge of the face adds nothing: the line is
    /// one the boundary already runs along, and the face comes back whole.
    #[test]
    fn a_cut_along_an_edge_the_face_already_has_leaves_the_face_alone() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(&the_top_of_a_cube(&model, cube), &[[1.0, 0.0, 0.0, 0.0]]);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].wires[0].len(), 4);
        assert_eq!(twice_the_area_of(&pieces[0]), 8.0);
    }

    /// A `Point` is plain data and may hold a coordinate that is not a number —
    /// `Plane::transformed` overflows a solid placed far enough out into one —
    /// so where a walk says a closed crossing starts is checked and not
    /// trusted. Without the check the box round it comes out as
    /// `Interval::new(NaN, NaN)` and the failure is reported against the reach.
    #[test]
    #[should_panic(expected = "where it starts is a real place")]
    fn a_closed_crossing_that_starts_nowhere_is_a_bug_and_not_a_box() {
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );
        where_the_crossing_starts(&flat, &flat, Point::new(f64::INFINITY, 0.0, 0.0));
    }
}

#[cfg(test)]
mod curved {
    use rcad_geom::{Isometry, Scale, Surface, Vector};
    use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};

    use super::super::MeetPoint;
    use super::super::bake::{BakedFace, BakedSolid, bake};
    use super::super::cuts;
    use super::{Piece, split_a_face};
    use crate::Model;

    /// A block eight across sitting on the origin with a round bar of radius
    /// one straight through the middle of it, both already cut against each
    /// other.
    ///
    /// §31.7 criterion 11's headline shape, one step short of being fused. The
    /// bar sticks out two either side, so every one of the block's six faces and
    /// every one of the bar's three has something to say.
    fn a_block_and_a_bar_through_it() -> Vec<BakedSolid> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let block = model.add_cube(8.0, &policy, &mut log).expect("a block");
        let bar = model
            .add_round_bar(
                1.0,
                12.0,
                &Isometry::translation(Vector::new(4.0, 4.0, -2.0)).expect("a shift"),
            )
            .expect("a bar through the middle");
        let mut solids = bake(&model, &[block, bar], &policy, &mut log).expect("both bake");
        cuts::gather(&mut solids, &policy, &mut log).expect("they cut each other");
        assert!(log.is_empty(), "nothing here had to be approximated");
        solids
    }

    fn split(face: &BakedFace) -> Vec<Piece> {
        let mut log = ProvenanceLog::new();
        let pieces = split_a_face(face, &Policy::certified(), &mut log)
            .expect("a lattice-aligned pair splits with nothing left undecided");
        assert!(log.is_empty(), "nothing here had to be approximated");
        pieces
    }

    /// Which surface a face sits on, as one word, for reading a table.
    fn kind(surface: &Surface) -> &'static str {
        match surface {
            Surface::Plane(_) => "plane",
            Surface::Sphere(_) => "ball",
            Surface::Cylinder(_) => "pipe",
            Surface::Cone(_) => "cone",
            Surface::Torus(_) => "ring",
            other => panic!("no {other:?} here"),
        }
    }

    /// §31.7 criterion 12: the pieces a face is cut into cover it and overlap
    /// nowhere.
    ///
    /// The oracle needs no area and so works on a curved face exactly as it does
    /// on a flat one: **every side of every piece is walked once, and a side two
    /// pieces share is walked once each way.** A piece that overlapped another
    /// would walk a side the same way twice; a piece missing would leave a side
    /// walked once with nothing walking it back, and the only sides allowed that
    /// are the ones on the rim the face came in with.
    ///
    /// A side is named by where it starts, where it stops and what carries it —
    /// never by a coordinate, because two ends that are boxes are compared as
    /// what the arrangement *called* them and not as what they rounded to.
    #[test]
    fn the_pieces_a_curved_face_is_cut_into_cover_it_and_overlap_nowhere() {
        let solids = a_block_and_a_bar_through_it();
        let mut curved_faces = 0;
        for solid in &solids {
            for face in &solid.faces {
                let rim: Vec<Surface> = face
                    .wires
                    .iter()
                    .flatten()
                    .map(|side| side.across)
                    .collect();
                let mut walked: Vec<(String, String, Surface)> = Vec::new();
                for piece in &split(face) {
                    for wire in &piece.wires {
                        for step in 0..wire.len() {
                            assert_eq!(
                                wire[step].ends[1].point(),
                                wire[(step + 1) % wire.len()].ends[0].point(),
                                "a side of a piece starts where the one before it stopped"
                            );
                        }
                        for side in wire {
                            walked.push((
                                name_of(&side.ends[0]),
                                name_of(&side.ends[1]),
                                side.carried_by[1],
                            ));
                        }
                    }
                }
                for side in &walked {
                    let how_often = walked.iter().filter(|other| *other == side).count();
                    let on_the_rim = rim.contains(&side.2);
                    // A side that closes on itself starts and stops at the same
                    // corner, so the two pieces either side of it walk what looks
                    // like the very same side. Nothing tells them apart but how
                    // many there are, and that is the whole check here.
                    if side.0 == side.1 {
                        assert_eq!(
                            how_often,
                            if on_the_rim { 1 } else { 2 },
                            "a closed side of a piece of a {} is walked the wrong number of times",
                            kind(&face.surface)
                        );
                        continue;
                    }
                    assert_eq!(
                        how_often,
                        1,
                        "one side of one piece of a {} is walked twice the same way",
                        kind(&face.surface)
                    );
                    let back = (side.1.clone(), side.0.clone(), side.2);
                    assert!(
                        walked.contains(&back) || on_the_rim,
                        "a side nobody walks back along has to be on the rim of the face"
                    );
                }
                if face.surface.as_plane().is_none() {
                    curved_faces += 1;
                }
            }
        }
        assert_eq!(curved_faces, 1, "the bar's wall is the curved face here");
    }

    /// What the arrangement calls one end of a side.
    ///
    /// The two ends of a side that two pieces share are the *same* corner and
    /// the arrangement gives it one name, so comparing names is exact where
    /// comparing the boxes they landed in would be a tolerance.
    fn name_of(end: &MeetPoint) -> String {
        match end {
            MeetPoint::Existing { vertex, .. } => format!("{vertex:?}"),
            MeetPoint::OfThreeSurfaces {
                surfaces, which, ..
            } => format!("{surfaces:?}/{which}"),
            MeetPoint::WhereAClosedCurveStarts { carried_by, .. } => format!("{carried_by:?}"),
            MeetPoint::APoleOfAClosedSurface {
                surface,
                at_the_top,
                ..
            } => format!("{surface:?}/pole {at_the_top}"),
        }
    }

    /// What each face of the pair comes apart into, face by face.
    ///
    /// The table is the thing to read: a block's face has a round hole in it for
    /// the first time in this kernel, and a bar's wall comes apart into three
    /// rings — below the block, inside it, and above — each bounded by two
    /// closed curves and neither of them an outer wire (§31.3).
    #[test]
    fn a_block_with_a_bar_through_it_comes_apart_into_a_ring_and_a_disk_and_three_tubes() {
        let solids = a_block_and_a_bar_through_it();
        let mut table: Vec<(&'static str, usize, Vec<usize>)> = Vec::new();
        for solid in &solids {
            for face in &solid.faces {
                let pieces = split(face);
                let mut wires: Vec<usize> = pieces.iter().map(|piece| piece.wires.len()).collect();
                wires.sort_unstable();
                table.push((kind(&face.surface), face.cuts.len(), wires));
            }
        }
        table.sort();

        assert_eq!(
            table,
            [
                // The bar's wall, cut by the block's top and bottom into three
                // rings, each between two closed curves, and by the bar's own
                // two caps, which cut nothing off it.
                ("pipe", 4, vec![2, 2, 2]),
                // The bar's two caps, cut by the block's four walls, which are
                // nowhere near them, and by the bar's own wall.
                ("plane", 5, vec![1]),
                ("plane", 5, vec![1]),
                // The block's top and bottom: a square with a round hole in it,
                // and the disk that came out. The other four cuts are the
                // block's own walls, along the edges of the square.
                ("plane", 5, vec![1, 2]),
                ("plane", 5, vec![1, 2]),
                // The block's four walls: cut by the bar's two cap planes, which
                // reach right past them and cut nothing off, and by the block's
                // own top, bottom and two walls beside them.
                ("plane", 6, vec![1]),
                ("plane", 6, vec![1]),
                ("plane", 6, vec![1]),
                ("plane", 6, vec![1]),
            ]
        );
    }

    /// A closed crossing with no corner on it still gets one, and it is the same
    /// one whichever face asks (§31.3, §34.4).
    ///
    /// The circle where the bar's wall cuts the block's top has no corner
    /// anywhere on it, and a wire needs a coedge, an edge needs two ends, and a
    /// wire that closes on one edge needs both ends to be one vertex.
    #[test]
    fn a_closed_cut_with_no_corner_on_it_starts_where_the_crossing_starts() {
        let solids = a_block_and_a_bar_through_it();
        let top = solids[0]
            .faces
            .iter()
            .find(|face| {
                face.surface
                    .as_plane()
                    .is_some_and(|flat| flat.normal() == Vector::new(0.0, 0.0, 1.0))
            })
            .expect("the block has a top");

        let disk = split(top)
            .into_iter()
            .find(|piece| piece.wires.len() == 1)
            .expect("the disk the bar took out");
        assert_eq!(disk.wires[0].len(), 1, "one closed side and nothing else");
        let side = &disk.wires[0][0];
        assert_eq!(side.ends[0].point(), side.ends[1].point());
        assert!(matches!(
            side.ends[0],
            MeetPoint::WhereAClosedCurveStarts { .. }
        ));
        assert_eq!(
            side.ends[0].point(),
            PredicatePoint::from([4.0, 5.0, 8.0]),
            "the point at angle zero on the circle of radius one about (4, 4), where angle zero \
             is whichever way `Plane::basis` faces and not whichever way looks natural"
        );
    }
}

#[cfg(test)]
mod what_the_curved_arrangement_reaches {
    use rcad_geom::{
        Axis, Cone, Cylinder, EdgeRun, Isometry, Plane, Point, Scale, Surface, SurfaceMeet, Torus,
        Vector, intersect, where_a_closed_crossing_starts,
    };
    use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog};

    use super::super::bake::bake;
    use super::super::cuts;
    use super::{
        HowItRuns, a_pair_that_crosses_in_straight_lines, a_plane_cuts_a_cone_in_rulings,
        a_plane_lies_along_a_pipes_axis, cross_of, dominant_axis_of_a_direction, gradient_at,
        how_a_crossing_runs, how_a_plane_leans_against_a_cone, round_a_closed_crossing,
        split_a_face, two_pipes_run_along_one_line, where_a_cones_apex_stands_against_a_plane,
    };
    use crate::{Error, Model, SolidKey};

    /// What every face of a pair comes apart into, as one word per face: the
    /// surface it sits on, how many surfaces cut it, and how many wires each
    /// piece has.
    ///
    /// A refusal is a row like any other, which is the point of a table (§30.5):
    /// what this milestone reaches and what it does not are read side by side.
    fn what_it_cuts(model: &Model, one: SolidKey, other: SolidKey) -> Vec<String> {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let mut solids = match bake(model, &[one, other], &policy, &mut log) {
            Ok(solids) => solids,
            Err(why) => return vec![format!("baking refused {why:?}")],
        };
        if let Err(why) = cuts::gather(&mut solids, &policy, &mut log) {
            return vec![format!("gathering refused {why:?}")];
        }
        let mut said = Vec::new();
        for solid in &solids {
            for face in &solid.faces {
                let sits_on = match face.surface {
                    Surface::Plane(_) => "plane",
                    Surface::Sphere(_) => "ball",
                    Surface::Cylinder(_) => "pipe",
                    Surface::Cone(_) => "cone",
                    Surface::Torus(_) => "ring",
                    other => panic!("no {other:?} here"),
                };
                said.push(match split_a_face(face, &policy, &mut log) {
                    Ok(pieces) => format!(
                        "{sits_on} cut by {} -> {:?}",
                        face.cuts.len(),
                        pieces
                            .iter()
                            .map(|piece| piece.wires.len())
                            .collect::<Vec<_>>()
                    ),
                    Err(Error::Undecidable { .. }) => format!("{sits_on} could not be decided"),
                    Err(why) => format!("{sits_on} refused {why}"),
                });
            }
        }
        said.sort();
        said
    }

    fn a_block(model: &mut Model) -> SolidKey {
        model
            .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a block eight across")
    }

    /// A round bar sitting on a block's top, cut off by it: §31.7 criterion 11's
    /// second half, where the hole does not go all the way through.
    ///
    /// The wall of the bar is cut by the block's top into what is inside the
    /// block and what is above it, and the block's *bottom* is cut too — a cut
    /// is by a surface and a cylinder reaches past the bar carrying it, which is
    /// what keeps the two faces either side of an edge split at the same place
    /// (§17.4, one word changed).
    #[test]
    fn a_blind_hole_cuts_the_face_it_stops_short_of_as_well() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = a_block(&mut model);
        let bar = model
            .add_round_bar(
                1.0,
                5.0,
                &Isometry::translation(Vector::new(4.0, 4.0, 5.0)).expect("a shift"),
            )
            .expect("a bar standing on the block's top");

        assert_eq!(
            what_it_cuts(&model, block, bar),
            [
                "pipe cut by 4 -> [2, 2]",
                "plane cut by 5 -> [1]",
                "plane cut by 5 -> [1]",
                "plane cut by 5 -> [2, 1]",
                "plane cut by 5 -> [2, 1]",
                "plane cut by 6 -> [1, 1]",
                "plane cut by 6 -> [1, 1]",
                "plane cut by 6 -> [1, 1]",
                "plane cut by 6 -> [1, 1]",
            ]
        );
    }

    /// A ball resting on a block: the first curved face this kernel cuts that is
    /// not a pipe, and the first with no boundary of its own at all.
    ///
    /// A ball's one wire is its seam and a seam trims nothing, so the face is
    /// the whole surface and every cell the cut leaves is inside it. There is no
    /// outer wire to read that off and §31.3 says there does not have to be.
    #[test]
    fn a_ball_cut_by_a_plane_comes_apart_into_two_caps() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = a_block(&mut model);
        let ball = model
            .add_ball(
                3.0,
                &Isometry::translation(Vector::new(4.0, 4.0, 8.0)).expect("a shift"),
            )
            .expect("a ball sitting on the block's top");

        assert_eq!(
            what_it_cuts(&model, block, ball),
            [
                "ball cut by 1 -> [1, 1]",
                "plane cut by 4 -> [1]",
                "plane cut by 4 -> [1]",
                "plane cut by 4 -> [1]",
                "plane cut by 4 -> [1]",
                "plane cut by 4 -> [1]",
                "plane cut by 5 -> [2, 1]",
            ]
        );
    }

    /// A bar turned about an awkward axis, so that every curve it cuts on the
    /// block is an ellipse and every curve the block cuts on it is one too.
    ///
    /// Nothing here is exactly representable and none of it needed a policy —
    /// the corners of a placed curved solid are boxes, and a box is not slop
    /// anybody failed to prove (§17.2).
    #[test]
    fn a_turned_bar_through_a_block_cuts_ellipses_and_nothing_has_to_be_approximated() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = a_block(&mut model);
        let axis = Axis::new(Point::new(4.0, 4.0, 4.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        let bar = model
            .add_round_bar(
                1.0,
                20.0,
                &Isometry::rotation(&axis, 0.7)
                    .expect("a rotation")
                    .compose(&Isometry::translation(Vector::new(4.0, 4.0, -6.0)).expect("a shift")),
            )
            .expect("a bar leaning through the block");

        assert_eq!(
            what_it_cuts(&model, block, bar),
            [
                "pipe cut by 8 -> [2, 1, 1, 2, 2]",
                "plane cut by 7 -> [1, 1]",
                "plane cut by 7 -> [1, 1]",
                "plane cut by 7 -> [1]",
                "plane cut by 7 -> [1]",
                "plane cut by 7 -> [1]",
                "plane cut by 7 -> [1]",
                "plane cut by 7 -> [2, 1]",
                "plane cut by 7 -> [2, 1]",
            ]
        );
    }

    /// §30.8 item 10, closed: a bar's wall with nothing cutting it comes back as
    /// itself, bounded by its two cap circles and no seam.
    ///
    /// A **closed edge has no direction rule** in the model — the mesher picks
    /// one by the sign of its turn round the face, which works for a cap circle
    /// and says nothing about a hole — and the arrangement cannot do without
    /// one: the two cap circles of a wall have to run *opposite* ways round the
    /// cylinder or the wall between them is not bounded at all.
    ///
    /// §31.3 settles the rule and it costs no arithmetic. Write `d` for the
    /// crossing's own direction `∇S × ∇T` and `o` for the face's own gradient
    /// `∇S`; then `(o × d)·∇T = (∇S·∇T)² − |∇S|²|∇T|²`, which Cauchy–Schwarz
    /// makes never positive and equal to `−|∇S × ∇T|²` exactly. So **left of
    /// forward is always the negative side of the partner**, everywhere, with no
    /// case analysis — and a wire walks a closed edge forward exactly when the
    /// face's material is inside the partner, which is what a corner off that
    /// partner already says.
    ///
    /// The wall's bottom cap has its normal pointing down and its top cap up, so
    /// the wall's material is on the negative side of both and it walks both
    /// forward; the two forward directions are `radial × down` and `radial × up`
    /// and they are opposite. That is the whole of it, and what this test says
    /// is that the winding count reads it: one piece, two wires, no seam.
    #[test]
    fn a_bar_wall_is_bounded_by_two_cap_circles_that_run_opposite_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bar = model
            .add_round_bar(1.0, 5.0, &Isometry::identity())
            .expect("a bar");
        let solids = bake(
            &model,
            &[bar],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar bakes");
        let wall = solids[0]
            .faces
            .iter()
            .find(|face| matches!(face.surface, Surface::Cylinder(_)))
            .expect("the bar has a wall");

        let pieces = split_a_face(wall, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a wall with nothing cutting it is itself");

        assert_eq!(pieces.len(), 1, "nothing cuts it, so it is one piece");
        assert_eq!(
            pieces[0].wires.len(),
            2,
            "bounded by its two cap circles, and by no seam"
        );
        for wire in &pieces[0].wires {
            assert_eq!(wire.len(), 1, "one closed side each");
            assert!(matches!(wire[0].carried_by[1], Surface::Plane(_)), "a cap");
            assert_eq!(
                wire[0].ends[0].point(),
                wire[0].ends[1].point(),
                "a side that closes on itself"
            );
        }
    }

    /// A ball with a pipe through it, which is the shape §34.9 says this lane
    /// meets its own refusal on, and it does.
    ///
    /// A ball and a pipe cross in a **quartic**, and ordering points along one
    /// takes the marcher rather than a determinant. It closes, so a place to
    /// start on it can be found — and reading that as "so it can be ordered" is
    /// the mistake worth naming: it gave the ball four pieces where it has
    /// three, silently, until the two questions were told apart.
    ///
    /// A **gap** (§16.10): the pieces are there. What closes it is
    /// `which_comes_first_round_a_crossing` learning to order a marched loop.
    #[test]
    fn a_ball_with_a_pipe_through_it_is_refused_rather_than_cut_wrongly() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let ball = model
            .add_ball(3.0, &Isometry::identity())
            .expect("a ball of radius three");
        let bar = model
            .add_round_bar(
                1.0,
                12.0,
                &Isometry::translation(Vector::new(0.0, 0.0, -6.0)).expect("a shift"),
            )
            .expect("a bar right through it");

        let said = what_it_cuts(&model, ball, bar);
        assert!(
            said.iter()
                .any(|row| row == "ball refused nothing can put points in order along this curve"),
            "expected the quartic to be refused by name, got {said:?}"
        );
        assert!(
            said.iter().any(|row| row.starts_with("pipe refused")),
            "and the wall of the bar with it, got {said:?}"
        );
    }

    /// The three plane-and-torus crossings a capped bend of pipe has, ordered
    /// rather than refused.
    ///
    /// A bend carries exactly these: its two caps are planes **through** the
    /// ring's axis, and a slab laid across it cuts the ring in the **two
    /// circles round the axis** — the outer one and the inner one the bend's
    /// seam runs along. `a_pair_that_crosses_in_a_conic` listed seven ordered
    /// pairs and plane-and-torus was not one, so every one of these came back
    /// `ACrossingThisCannotBeOrderedAlong` and a bend could not be fused with
    /// anything at all, itself included.
    ///
    /// The second half is what says the first is not vacuous. Reaching
    /// [`HowItRuns::RoundAClosedCurve`] only means the gate opened; what says
    /// the ordering *works* is that two points on the curve come back decided
    /// both ways round, and that the two circles round the axis come back
    /// **opposite** — the inner one winds about the plane's negated normal, and
    /// a row that answered `CannotTell` or handed back `+n` for both would
    /// order a bend's seam backwards instead of refusing.
    #[test]
    fn the_three_plane_and_torus_crossings_a_bend_has_are_ordered_rather_than_refused() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let ring = Surface::Torus(
            Torus::new(Axis::z(), 4.0, 1.0).expect("a ring of radius four with a tube of one"),
        );

        let cap = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("a cap plane holding the ring's axis"),
        );
        let across = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("a slab face square to the ring's axis"),
        );

        // The meridian circles are centred at `(±4, 0, 0)` with radius one; the
        // circles round the axis have radius `4 ± 1` in the plane `z = 0`.
        for (named, flat, on_the_curve) in [
            ("a cap through the axis", &cap, [4.0, 0.0, 1.0]),
            ("a slab square to the axis", &across, [5.0, 0.0, 0.0]),
        ] {
            let SurfaceMeet::Curves(runs) = intersect(flat, &ring, &policy, &mut log)
                .unwrap_or_else(|why| panic!("{named} cuts the ring in circles: {why:?}"))
            else {
                panic!("{named} cuts the ring in two curves and nothing else");
            };
            assert_eq!(runs.len(), 2, "{named} cuts the ring in two circles");
            for run in &runs {
                let at = PredicatePoint::from(on_the_curve);
                let ran = how_a_crossing_runs(flat, &ring, run, at, &mut log)
                    .unwrap_or_else(|why| panic!("{named} could not be ordered: {why:?}"));
                assert!(
                    matches!(ran, HowItRuns::RoundAClosedCurve { .. }),
                    "{named} runs round a closed curve, not along an axis"
                );
            }
        }

        // Both circles round the axis, ordered from the same angle round the
        // same axis: quarter of a turn on, then half a turn on.
        let ordered_round_the_axis = |radius: f64| {
            let point = |angle: f64| {
                PredicatePoint::from([radius * angle.cos(), radius * angle.sin(), 0.0])
            };
            round_a_closed_crossing(
                [&across, &ring],
                point(0.0),
                point(core::f64::consts::FRAC_PI_2),
                point(core::f64::consts::PI),
            )
            .expect("a circle round the axis is ordered")
        };
        let outer = ordered_round_the_axis(5.0);
        let inner = ordered_round_the_axis(3.0);
        assert_eq!(outer, Decision::Pos, "the outer circle winds about `+n`");
        assert_eq!(inner, Decision::Neg, "and the inner one about `-n`");
    }

    /// The two rulings a plane down a cone's axis cuts are ordered along an
    /// axis, and the ellipse a plane across it cuts still runs round a closed
    /// curve.
    ///
    /// Both used to end in `ACrossingThisCannotBeOrderedAlong`: the straight
    /// gate asked only about a plane and a *pipe*, so a ruling fell past it into
    /// the conic gate, which is `false` for a pair that does not close. A ruling
    /// is an endless straight line and is ordered the way a plane along a pipe's
    /// axis already was.
    ///
    /// **The tangent plane is left out on purpose and the last block says why.**
    #[test]
    fn the_two_rulings_a_plane_through_a_cones_apex_cuts_are_ordered_along_an_axis() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let cone = Surface::Cone(Cone::new(Axis::z(), 0.5).expect("a cone leaning a half"));
        let down_the_axis = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("a plane holding the cone's axis"),
        );

        let SurfaceMeet::Curves(runs) = intersect(&down_the_axis, &cone, &policy, &mut log)
            .expect("a plane down a cone's axis cuts it in two rulings")
        else {
            panic!("a plane down a cone's axis cuts curves and nothing else");
        };
        assert_eq!(runs.len(), 2, "it cuts two rulings, one either side");

        // `(2, 0, 4)` is on the cone — `2² = (4/2)²` — and on the plane `y = 0`.
        let at = PredicatePoint::from([2.0, 0.0, 4.0]);
        for run in &runs {
            let ran = how_a_crossing_runs(&down_the_axis, &cone, run, at, &mut log)
                .expect("a ruling is ordered along the axis it climbs fastest");
            let HowItRuns::AlongAnAxis { axis, .. } = ran else {
                panic!("a ruling runs on for ever, so it is not ordered round a closed curve");
            };
            assert_eq!(axis, 2, "the ruling climbs `z` twice as fast as `x`");
        }

        // The ellipse is the row the straight gate must not swallow.
        let across = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
                .expect("a plane square to the cone's axis"),
        );
        let SurfaceMeet::Curves(circle) = intersect(&across, &cone, &policy, &mut log)
            .expect("a plane across a cone cuts one closed curve")
        else {
            panic!("a plane across a cone cuts curves and nothing else");
        };
        assert!(
            matches!(
                how_a_crossing_runs(
                    &across,
                    &cone,
                    &circle[0],
                    PredicatePoint::from([2.0, 0.0, 4.0]),
                    &mut log
                )
                .expect("a circle round a cone is ordered"),
                HowItRuns::RoundAClosedCurve { .. }
            ),
            "a plane cutting across a cone still runs round a closed curve"
        );

        // **A tangent plane's one ruling is still refused, and by the same gap
        // as a plane resting on a pipe.** The gate lets it through now, and the
        // line under the gate is `∇S × ∇Q`, which for a plane lying along a
        // cone's own side is exactly nothing — the two gradients are parallel
        // everywhere on the ruling. `how_a_crossing_runs`'s own note names the
        // four families of fix and why the working one is not a small change.
        let touching = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.5))
                .expect("a plane resting along one ruling"),
        );
        let SurfaceMeet::Curves(ruling) = intersect(&touching, &cone, &policy, &mut log)
            .expect("a tangent plane touches the cone along one ruling")
        else {
            panic!("a tangent plane touches along a curve and nothing else");
        };
        assert!(
            matches!(
                how_a_crossing_runs(
                    &touching,
                    &cone,
                    &ruling[0],
                    PredicatePoint::from([2.0, 0.0, 4.0]),
                    &mut log
                ),
                Err(Error::Undecidable { .. })
            ),
            "the direction of a tangent plane's ruling is `∇S × ∇Q`, which is nothing"
        );

        // `.is_err()` used to be the whole of the line above, and it would have
        // passed just as well had the straight gate turned the pair away — which
        // is `ACrossingThisCannotBeOrderedAlong` and a different sentence. The
        // gate lets it through, and what gives up is the direction.
        assert!(
            a_pair_that_crosses_in_straight_lines(&touching, &cone, &mut log)
                .expect("a plane and a cone is a row nothing can run out of arithmetic on"),
            "a plane resting along a cone's own side cuts one endless ruling"
        );
        assert_eq!(
            cross_of(
                gradient_at(&touching, PredicatePoint::from([2.0, 0.0, 4.0])),
                gradient_at(&cone, PredicatePoint::from([2.0, 0.0, 4.0])),
            )
            .map(Interval::sign),
            [Decision::Zero; 3],
            "the two gradients are parallel, so the cross product is exactly nothing"
        );
    }

    /// **Two pipes side by side cut straight lines, and both of them are
    /// ordered up the common axis.**
    ///
    /// Two pipes of radius five whose axes both run up `z`, six apart, meet
    /// along the two lines through `(3, ±4, z)` — a 3-4-5 on both, so the
    /// meeting is the real one and not a near one. `intersect` builds both, and
    /// `where_two_pipes_side_by_side_cross`'s own doc says every answer it gives
    /// is straight.
    ///
    /// `∇S × ∇Q` at `(3, 4, 0)` is `(3, 4, 0) × (−3, 4, 0) = (0, 0, 24)`, which
    /// is the common axis with nothing cancelling, so
    /// `dominant_axis_of_a_direction` names `z` and the sign is up it. This used
    /// to be `Err(ACrossingThisCannotBeOrderedAlong)` and the gap §48 pinned:
    /// the pair fell past a straight gate that said `false` for it into a conic
    /// gate that says `false` for anything that does not close.
    #[test]
    fn two_pipes_side_by_side_run_up_their_common_axis() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let here = Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a pipe of radius five"));
        let there = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(6.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                    .expect("an axis six along `x`"),
                5.0,
            )
            .expect("a second pipe of the same radius"),
        );

        let SurfaceMeet::Curves(runs) = intersect(&here, &there, &policy, &mut log)
            .expect("two pipes of radius five, six apart, meet")
        else {
            panic!("two pipes side by side meet along curves and nothing else");
        };
        assert_eq!(
            runs.len(),
            2,
            "one line either side of the plane of the axes"
        );

        let at = PredicatePoint::from([3.0, 4.0, 0.0]);
        assert_eq!(
            cross_of(gradient_at(&here, at), gradient_at(&there, at)),
            [
                Interval::exactly(0.0),
                Interval::exactly(0.0),
                Interval::exactly(24.0),
            ],
            "the direction is there, exactly, and it is the common axis"
        );
        assert!(
            a_pair_that_crosses_in_straight_lines(&here, &there, &mut log)
                .expect("two axes both stored as `z` are proved along one line"),
            "two pipes whose axes run one way cut endless lines"
        );
        for run in &runs {
            assert!(
                matches!(
                    how_a_crossing_runs(&here, &there, run, at, &mut log),
                    Ok(HowItRuns::AlongAnAxis {
                        axis: 2,
                        forward_climbs_the_axis: true
                    })
                ),
                "the line runs up `z`, and `∇S × ∇Q` points up it"
            );
        }
    }

    /// **Two pipes resting against each other have no direction to read, the
    /// same way a plane resting on a pipe has none.**
    ///
    /// Two pipes of radius five whose axes are ten apart touch along the one
    /// ruling through `(5, 0, z)`. The axes are parallel, so the straight gate
    /// lets the pair through — and what gives up under it is `∇S × ∇Q`, which is
    /// `(10, 0, 0) × (−10, 0, 0)` and exactly nothing. Both gradients point
    /// along the line between the axes.
    ///
    /// So the refusal moved from [`Error::ACrossingThisCannotBeOrderedAlong`] to
    /// [`Error::Undecidable`], which is the limit `how_a_crossing_runs`'s own
    /// note names the four families of fix for. Closing the side-by-side gap
    /// does not close this and does not claim to.
    #[test]
    fn two_pipes_resting_against_each_other_have_no_direction_to_read() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let here = Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a pipe of radius five"));
        let there = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(10.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                    .expect("an axis ten along `x`"),
                5.0,
            )
            .expect("a second pipe of the same radius"),
        );

        let SurfaceMeet::Curves(runs) = intersect(&here, &there, &policy, &mut log)
            .expect("two pipes resting against each other touch")
        else {
            panic!("two pipes resting against each other touch along one ruling");
        };
        assert_eq!(runs.len(), 1, "one ruling and not two");

        let at = PredicatePoint::from([5.0, 0.0, 0.0]);
        assert_eq!(
            cross_of(gradient_at(&here, at), gradient_at(&there, at)).map(Interval::sign),
            [Decision::Zero; 3],
            "the two gradients are opposite, so the cross product is exactly nothing"
        );
        assert!(
            a_pair_that_crosses_in_straight_lines(&here, &there, &mut log)
                .expect("two axes both stored as `z` are proved along one line"),
            "the gate lets it through — the axes are parallel"
        );
        assert!(
            matches!(
                how_a_crossing_runs(&here, &there, &runs[0], at, &mut log),
                Err(Error::Undecidable { .. })
            ),
            "the direction of the shared ruling is `∇S × ∇Q`, which is nothing"
        );
    }

    /// **Two pipes whose `f64` cross product is exactly nothing and whose axes
    /// are not along one line.**
    ///
    /// The unit directions `(a, 0, c)` and `(b, 0, c)`, with `a` and `b` one ulp
    /// apart and `a·c` and `b·c` rounding to the same `f64`. The cross product's
    /// `y` component is `c·b − a·c`, which *rounds* to nothing and exactly is
    /// `(b − a)·c ≈ −5.1e-17`. The other two components are nothing for a reason
    /// that is not rounding: both directions have no `y` at all.
    ///
    /// So `w₁.cross(w₂) == Vector::zero()` — the test both doors used to run —
    /// said these two pipes are side by side. They are not: `intersect` names
    /// their crossing one closed quartic loop, and the gate saying `true` had
    /// [`how_a_crossing_runs`] order that loop along an axis as though it were
    /// an endless straight line. **A refusal turned into a silently wrong
    /// ordering**, which is worse than the refusal it replaced.
    ///
    /// `two_pipes_run_along_one_line` proves the `y` component's sign with
    /// [`orient2d`](rcad_predicates::orient2d) instead, so the pair comes back
    /// `Decision::Pos`, the gate answers `false`, and the crossing goes to the
    /// conic gate — which turns a quartic away by name.
    #[test]
    fn two_pipes_a_rounded_cross_product_calls_parallel_are_not() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let (a, b, c) = (
            0.393_507_725_793_881_44_f64,
            0.393_507_725_793_881_4_f64,
            0.919_321_309_304_058_f64,
        );
        assert_ne!(a, b, "the two directions are one ulp apart");
        assert_eq!(a * c, b * c, "and their products round to the same `f64`");

        let pipe = |origin: Point, along: Vector| {
            Cylinder::new(Axis::new(origin, along).expect("an axis"), 1.0).expect("a pipe")
        };
        let one = pipe(Point::origin(), Vector::new(a, 0.0, c));
        let other = pipe(Point::new(0.0, 1.0, 0.0), Vector::new(b, 0.0, c));
        assert_eq!(
            one.axis().direction().cross(other.axis().direction()),
            Vector::zero(),
            "the `f64` cross product is exactly nothing, and that is the trap"
        );

        assert_eq!(
            two_pipes_run_along_one_line(&one, &other),
            Decision::Pos,
            "kept as three determinants, the `y` component is proved below zero"
        );

        let (here, there) = (Surface::Cylinder(one), Surface::Cylinder(other));
        let SurfaceMeet::Curves(runs) = intersect(&here, &there, &policy, &mut log)
            .expect("two leaning pipes a unit apart cross")
        else {
            panic!("two leaning pipes cross along curves and nothing else");
        };
        assert_eq!(
            runs.len(),
            1,
            "one closed loop right round where they cross"
        );

        assert!(
            !a_pair_that_crosses_in_straight_lines(&here, &there, &mut log)
                .expect("the axes are proved apart, so nothing ran out of arithmetic"),
            "a closed loop is not an endless straight line"
        );
        let at = PredicatePoint::from([1.0, 0.0, 0.0]);
        assert!(
            matches!(
                how_a_crossing_runs(&here, &there, &runs[0], at, &mut log),
                Err(Error::ACrossingThisCannotBeOrderedAlong)
            ),
            "a quartic is refused by name, where it used to be ordered along `y`"
        );
    }

    /// **A plane whose `f64` dot with a pipe's axis is exactly nothing and which
    /// does not hold that axis.**
    ///
    /// The pipe-and-pipe row above one row down. A stored normal and a stored
    /// axis direction whose three products sum to exactly `0.0` in `f64` while
    /// the exact sum is above zero.
    /// [`how_many_planes_a_rounded_dot_calls_flush_with_an_axis_are_not`] says
    /// how common that is; this one is the pair written out end to end.
    ///
    /// `intersect` decides the same sign exactly and names the crossing
    /// `RightRoundWhereItsFacesCross`: one closed ellipse. The rounded test said
    /// the plane lay along the axis, so [`how_a_crossing_runs`] ordered that
    /// ellipse along an axis as though it were an endless ruling.
    ///
    /// **What it gets now is a refusal and not an order.** The pair falls to the
    /// conic gate, which lets a plane and a pipe through, and
    /// `where_a_closed_crossing_starts` gives up — an ellipse this nearly along
    /// the axis is of order `1/(n·w)` long, and finding a place to start on it
    /// runs out of `f64`. `ACrossingThisCannotBeOrderedAlong` for a conic whose
    /// shape is known is a **gap** of the same kind §34.10 files, and it is the
    /// honest end of a pair that used to come back with an answer that was
    /// wrong.
    #[test]
    fn a_plane_a_rounded_dot_calls_flush_with_an_axis_is_not() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let normal = Vector::new(
            0.050_410_611_140_686_46,
            0.827_151_841_248_433_8,
            -0.559_712_963_762_452_5,
        );
        let along = Vector::new(
            0.566_485_713_199_844_4,
            -0.485_233_694_733_434_14,
            -0.666_064_710_246_537_6,
        );
        let flat = Plane::from_point_and_normal(Point::origin(), normal).expect("a plane");
        let pipe = Cylinder::new(Axis::new(Point::origin(), along).expect("an axis"), 1.0)
            .expect("a pipe of radius one");
        assert_eq!(
            flat.normal(),
            normal,
            "the plane stores the normal it was given, so the numbers below are the \
             ones the doors read"
        );
        assert_eq!(pipe.axis().direction(), along, "and so does the axis");
        assert_eq!(
            flat.normal().dot(pipe.axis().direction()),
            0.0,
            "the `f64` dot is exactly nothing, and that is what used to decide this row"
        );

        assert_eq!(
            a_plane_lies_along_a_pipes_axis(&flat, &pipe),
            Decision::Pos,
            "summed with no rounding thrown away, the dot is proved above zero"
        );

        let (here, there) = (Surface::Plane(flat), Surface::Cylinder(pipe));
        let SurfaceMeet::Curves(runs) = intersect(&here, &there, &policy, &mut log)
            .expect("a plane cutting across a pipe crosses it")
        else {
            panic!("a plane cutting a pipe meets it along curves and nothing else");
        };
        assert_eq!(runs.len(), 1, "one closed ellipse");
        assert_eq!(
            runs[0].run(),
            &EdgeRun::RightRoundWhereItsFacesCross,
            "`intersect` decides the same sign exactly, and names a curve that closes"
        );

        assert!(
            !a_pair_that_crosses_in_straight_lines(&here, &there, &mut log)
                .expect("the dot is proved above zero, so nothing ran out of arithmetic"),
            "an ellipse is not an endless straight line"
        );
        assert!(
            matches!(
                how_a_crossing_runs(
                    &here,
                    &there,
                    &runs[0],
                    PredicatePoint::from([0.0; 3]),
                    &mut log
                ),
                Err(Error::ACrossingThisCannotBeOrderedAlong)
            ),
            "the pair goes to the conic gate, which lets a plane and a pipe through, \
             and what gives up is finding a place to start on an ellipse this long"
        );
    }

    /// **How many planes the rounded dot called flush with a pipe's axis and the
    /// exact one does not, and what `intersect` says about each.**
    ///
    /// A search, not a bound: twenty thousand random axes, each given three
    /// normals built square to it in `f64` and then nudged by one ulp. The
    /// draws that survive are the ones both [`Axis::new`] and
    /// [`Plane::from_point_and_normal`] hand straight back, because only those
    /// let a reader write the numbers the doors read.
    ///
    /// Every disagreement is put to `intersect`, which decides the same sign
    /// exactly: it names a curve that **closes** every time, so the rounded test
    /// was wrong on all of them and never merely unlucky. Only a floor is
    /// asserted — the exact count moves with the seed, and what has to stay true
    /// is that this is a family and not a curiosity.
    #[test]
    fn how_many_planes_a_rounded_dot_calls_flush_with_an_axis_are_not() {
        let policy = Policy::certified();
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut draw = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        };

        let mut drawn = 0_usize;
        let mut disagreed = 0_usize;
        for _ in 0..20_000 {
            let Ok(axis) = Axis::new(Point::origin(), Vector::new(draw(), draw(), draw())) else {
                continue;
            };
            let along = axis.direction();
            if Axis::new(Point::origin(), along)
                .expect("an axis")
                .direction()
                != along
            {
                continue;
            }
            let [x, y, z] = along
                .cross(Vector::new(draw(), draw(), draw()))
                .components();
            for nudged in [
                Vector::new(f64::from_bits(x.to_bits() + 1), y, z),
                Vector::new(x, f64::from_bits(y.to_bits() + 1), z),
                Vector::new(x, y, f64::from_bits(z.to_bits() + 1)),
            ] {
                let Ok(flat) = Plane::from_point_and_normal(Point::origin(), nudged) else {
                    continue;
                };
                drawn += 1;
                if flat.normal().dot(along) != 0.0 {
                    continue;
                }
                let pipe = Cylinder::new(axis, 1.0).expect("a pipe of radius one");
                if a_plane_lies_along_a_pipes_axis(&flat, &pipe) == Decision::Zero {
                    continue;
                }
                disagreed += 1;
                let mut log = ProvenanceLog::new();
                let met = intersect(
                    &Surface::Plane(flat),
                    &Surface::Cylinder(pipe),
                    &policy,
                    &mut log,
                )
                .expect("a plane cutting across a pipe is a shape `intersect` names");
                assert!(
                    matches!(&met, SurfaceMeet::Curves(runs) if runs.len() == 1
                        && runs[0].run() == &EdgeRun::RightRoundWhereItsFacesCross),
                    "the rounded dot called this plane flush with the axis, and \
                     `intersect` says {met:?}"
                );
            }
        }

        println!(
            "{disagreed} of {drawn} near-square planes are called flush with the axis by a \
             rounded dot and are not"
        );
        assert!(
            disagreed * 10 > drawn,
            "these are meant to be a family and not a curiosity: {disagreed} of {drawn}"
        );
    }

    /// **A plane and a cone the two rounded tests together call two rulings, and
    /// whose crossing is one closed ellipse.**
    ///
    /// The third row of the same trap, and the one that takes **both** of its
    /// signs being wrong at once. The plane goes through the origin and the
    /// cone's apex is a single ulp off it: the `f64` distance from the plane to
    /// the apex is exactly nothing and the exact one is not. The plane is also
    /// drawn a rounding off tangent to the cone, so `f64`'s
    /// `up² − slope²·sideways` lands at or below nothing where the exact `E` is
    /// above it.
    ///
    /// Either sign right on its own and the pair is a curve. Both wrong
    /// together and the old gate said *two rulings meeting at the apex* about a
    /// crossing `intersect` names `RightRoundWhereItsFacesCross` — one closed
    /// ellipse — and [`how_a_crossing_runs`] ordered that ellipse **along an
    /// axis**, exactly as the two pipe rows above did before them.
    ///
    /// **What it gets now is the closed curve it is.** Both signs are proved,
    /// the apex comes back off the plane, the pair falls to the conic gate, and
    /// what orders it is `where_a_closed_crossing_starts` — a place to start on
    /// an ellipse this long is still findable here, so this one ends in an
    /// answer and not a refusal.
    #[test]
    fn a_rounded_apex_and_a_rounded_lean_together_call_a_closed_ellipse_two_rulings() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let normal = Vector::new(
            -0.363_397_689_122_298_75,
            -0.399_628_971_525_338,
            0.841_569_251_255_161_7,
        );
        let along = Vector::new(
            -0.426_496_054_832_981_4,
            0.483_888_527_748_997_2,
            0.764_168_180_392_778_8,
        );
        let apex = Point::new(
            -0.162_754_196_529_426_2,
            0.361_000_414_878_634_9,
            0.101_146_430_287_779_8,
        );
        let slope = 0.759_265_215_818_364_9;
        let flat = Plane::from_point_and_normal(Point::origin(), normal).expect("a plane");
        let cone = Cone::new(Axis::new(apex, along).expect("an axis"), slope).expect("a cone");
        assert_eq!(
            flat.normal(),
            normal,
            "the plane stores the normal it was given, so the numbers below are the \
             ones the doors read"
        );
        assert_eq!(cone.axis().direction(), along, "and so does the axis");
        assert_eq!(cone.apex(), apex, "and so does the apex");

        assert_eq!(
            flat.signed_distance_to(cone.apex()),
            0.0,
            "the `f64` distance to the apex is exactly nothing, and that is the first \
             thing that used to decide this row"
        );
        let up = normal.dot(along);
        let sideways = normal.cross(along).length_squared();
        assert!(
            up * up - slope * slope * sideways <= 0.0,
            "and `f64`'s own `E` is at or below nothing, which is the second"
        );

        assert_eq!(
            where_a_cones_apex_stands_against_a_plane(&flat, &cone),
            Decision::Pos,
            "summed with no rounding thrown away, the apex is proved off the plane"
        );
        assert_eq!(
            how_a_plane_leans_against_a_cone(&flat, &cone),
            Decision::Pos,
            "and `E` is proved above nothing, so the plane cuts across the cone"
        );
        assert_eq!(
            a_plane_cuts_a_cone_in_rulings(&flat, &cone),
            Some(false),
            "so the pair does not cut rulings, and nothing ran out of arithmetic saying so"
        );

        let (here, there) = (Surface::Plane(flat), Surface::Cone(cone));
        let SurfaceMeet::Curves(runs) =
            intersect(&here, &there, &policy, &mut log).expect("a plane cutting across a cone")
        else {
            panic!("a plane cutting across a cone meets it along curves and nothing else");
        };
        assert_eq!(runs.len(), 1, "one closed ellipse");
        assert_eq!(
            runs[0].run(),
            &EdgeRun::RightRoundWhereItsFacesCross,
            "`intersect` decides the same two signs exactly, and names a curve that closes"
        );

        assert!(
            !a_pair_that_crosses_in_straight_lines(&here, &there, &mut log)
                .expect("both signs are proved, so nothing ran out of arithmetic"),
            "an ellipse is not a pair of endless rulings"
        );
        let start = where_a_closed_crossing_starts(&runs[0], [&here, &there], &mut log)
            .expect("a place to start on the ellipse");
        assert!(
            matches!(
                how_a_crossing_runs(
                    &here,
                    &there,
                    &runs[0],
                    PredicatePoint::from(start.coordinates()),
                    &mut log
                ),
                Ok(HowItRuns::RoundAClosedCurve { .. })
            ),
            "the ellipse is ordered round the closed curve it is, where the rounded \
             gate had it ordered along an axis"
        );
    }

    /// **How many planes drawn a rounding off tangent to a cone the two rounded
    /// tests get wrong, what each one costs, and that the proved gate gets every
    /// one of them right.**
    ///
    /// A search, not a bound. Three families of draw, forty thousand each:
    ///
    /// 0. a plane through the origin with a **random** normal, and a cone whose
    ///    apex is nudged by one ulp until the `f64` distance to the plane reads
    ///    exactly nothing;
    /// 1. the same nudged apex, with the plane drawn a rounding off **tangent**
    ///    to the cone — the normal `−cos θ·û + sin θ·ŵ` of the plane that rests
    ///    along one ruling, which puts `E` within a rounding of nothing;
    /// 2. the near-tangent plane again with the apex left exactly **on** it, so
    ///    that `E` alone decides the row.
    ///
    /// Every draw is put to `intersect`, which decides the same two signs
    /// exactly and is the oracle. The rounded gate is written out here rather
    /// than called, because the one in the tree is the proved one now.
    ///
    /// Only floors are asserted — the counts move with the seed — and the one
    /// thing asserted exactly is that **the proved gate never disagrees with
    /// `intersect` about any of the 360 000 draws**.
    #[test]
    fn how_many_near_tangent_planes_a_rounded_cone_gate_gets_wrong() {
        let policy = Policy::certified();
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut draw = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        };

        let mut drawn = 0_usize;
        let mut a_closed_ellipse_called_rulings = 0_usize;
        let mut ordered_along_an_axis = 0_usize;
        let mut rulings_called_a_curve = 0_usize;
        let mut the_apex_alone_called_rulings = 0_usize;
        for round in 0..3 {
            for _ in 0..40_000 {
                let Ok(axis) = Axis::new(Point::origin(), Vector::new(draw(), draw(), draw()))
                else {
                    continue;
                };
                let along = axis.direction();
                let Ok(square_to_it) = Axis::new(
                    Point::origin(),
                    along.cross(Vector::new(draw(), draw(), draw())),
                ) else {
                    continue;
                };
                let half_angle = draw().abs() * 1.3 + 0.1;
                let slope = half_angle.tan();
                let leaning = if round == 0 {
                    Vector::new(draw(), draw(), draw())
                } else {
                    square_to_it.direction() * -half_angle.cos() + along * half_angle.sin()
                };
                let Ok(flat) = Plane::from_point_and_normal(Point::origin(), leaning) else {
                    continue;
                };
                let normal = flat.normal();
                let [x, y, z] = normal
                    .cross(Vector::new(draw(), draw(), draw()))
                    .components();
                let apexes = if round == 2 {
                    [Point::origin(); 3]
                } else {
                    [
                        Point::new(f64::from_bits(x.to_bits() + 1), y, z),
                        Point::new(x, f64::from_bits(y.to_bits() + 1), z),
                        Point::new(x, y, f64::from_bits(z.to_bits() + 1)),
                    ]
                };
                for apex in apexes {
                    let Ok(cone) = Cone::new(
                        Axis::new(apex, along).expect("an axis through the apex"),
                        slope,
                    ) else {
                        continue;
                    };
                    let up = normal.dot(along);
                    let sideways = normal.cross(along).length_squared();
                    let rounded = flat.signed_distance_to(apex) == 0.0
                        && up * up - slope * slope * sideways <= 0.0;
                    drawn += 1;

                    let mut log = ProvenanceLog::new();
                    let here = Surface::Plane(flat);
                    let there = Surface::Cone(cone);
                    let met = intersect(&here, &there, &policy, &mut log);
                    let proved = a_plane_cuts_a_cone_in_rulings(&flat, &cone);
                    match &met {
                        Ok(SurfaceMeet::TouchAtOnePoint { .. }) => {
                            assert_eq!(
                                proved,
                                Some(false),
                                "two surfaces sharing only the apex do not share a ruling"
                            );
                            the_apex_alone_called_rulings += usize::from(rounded);
                        }
                        Ok(SurfaceMeet::Curves(runs))
                            if runs[0].run() == &EdgeRun::RightRoundWhereItsFacesCross =>
                        {
                            assert_eq!(
                                proved,
                                Some(false),
                                "a closed ellipse is not a pair of endless rulings"
                            );
                            if !rounded {
                                continue;
                            }
                            a_closed_ellipse_called_rulings += 1;
                            let Ok(start) =
                                where_a_closed_crossing_starts(&runs[0], [&here, &there], &mut log)
                            else {
                                continue;
                            };
                            // The rounded gate is what `a_pair_that_crosses_in_
                            // straight_lines` used to answer, so this is what
                            // `how_a_crossing_runs` did with the ellipse.
                            let at = PredicatePoint::from(start.coordinates());
                            ordered_along_an_axis += usize::from(
                                dominant_axis_of_a_direction(
                                    cross_of(gradient_at(&here, at), gradient_at(&there, at)),
                                    &mut log,
                                )
                                .is_ok(),
                            );
                        }
                        // Every other named shape is straight or bending, and
                        // `intersect` refusing to name which piece a run is says
                        // nothing about the two signs. Only the rounded gate's
                        // second direction is counted here, and it is counted
                        // against the proved gate rather than against the shape.
                        _ => {
                            if proved == Some(true) && !rounded {
                                rulings_called_a_curve += 1;
                            }
                        }
                    }
                }
            }
        }

        println!(
            "{drawn} draws: {a_closed_ellipse_called_rulings} closed ellipses the rounded \
             gate called rulings, {ordered_along_an_axis} of them with a direction to be \
             ordered along; {the_apex_alone_called_rulings} apexes-alone called rulings; \
             {rulings_called_a_curve} rulings called a curve"
        );
        assert!(
            a_closed_ellipse_called_rulings > 1_000,
            "a closed curve named an endless line is meant to be a family and not a \
             curiosity: {a_closed_ellipse_called_rulings} of {drawn}"
        );
        assert_eq!(
            ordered_along_an_axis, a_closed_ellipse_called_rulings,
            "every one of them had a direction, so every one of them was ordered along \
             an axis rather than refused"
        );
        assert!(
            rulings_called_a_curve > 1_000,
            "and the other way round is a family too: {rulings_called_a_curve} of {drawn}"
        );
    }
}

#[cfg(test)]
mod how_often_cutting_a_curved_face_refuses {
    use std::collections::BTreeMap;

    use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
    use rcad_predicates::{Policy, ProvenanceLog};

    use super::super::bake::bake;
    use super::super::cuts;
    use super::split_a_face;
    use crate::{Error, Model};

    /// How many pairs each family draws.
    const DRAWS: usize = 256;

    /// A deterministic sequence, so that a rate is one number and not a range.
    ///
    /// Nothing here is trying to be random. What it has to be is *the same every
    /// run*, because a share of refusals that moves between runs is a share
    /// nobody can put a threshold under (§17.7).
    struct Draw(u64);

    impl Draw {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            self.0 >> 33
        }

        /// A whole number in `low..=high`.
        fn whole(&mut self, low: i64, high: i64) -> f64 {
            let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
            let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
            picked as f64
        }

        /// A number between zero and one, to the nearest thousandth.
        fn fraction(&mut self) -> f64 {
            (self.next() % 1000) as f64 / 1000.0
        }
    }

    /// How each family writes its pair down.
    #[derive(Clone, Copy)]
    enum Family {
        /// Whole numbers, axes square to each other.
        Lattice,
        /// The whole pair turned about an awkward axis.
        Turned,
        /// The block square and the bar leaning.
        Leaning,
        /// The lattice pair with every number multiplied by a power of two.
        AtAPowerOfTwo,
    }

    impl Family {
        fn named(self) -> &'static str {
            match self {
                Family::Lattice => "lattice",
                Family::Turned => "turned",
                Family::Leaning => "leaning",
                Family::AtAPowerOfTwo => "written at 2^k",
            }
        }
    }

    /// A block with a round bar somewhere through it, written the family's way.
    fn a_block_and_a_bar(family: Family, draw: &mut Draw) -> Option<(Model, [crate::SolidKey; 2])> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let scale = match family {
            Family::AtAPowerOfTwo => 2.0_f64.powi(i32::try_from(draw.next() % 9).ok()? - 4),
            Family::Lattice | Family::Turned | Family::Leaning => 1.0,
        };
        let side = 8.0 * scale;
        // The bar is kept a whole unit clear of the block's walls. A bar exactly
        // touching one is a plane tangent to a cylinder, which is roadmap §7.4's
        // own catalogue entry and not what a *control* family is for — a control
        // that draws knife edges measures the knife edges (§17.7).
        let radius = draw.whole(1, 2);
        let at = [
            draw.whole(1 + radius as i64, 7 - radius as i64) * scale,
            draw.whole(1 + radius as i64, 7 - radius as i64) * scale,
        ];
        let radius = radius * scale;
        let block = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .ok()?;
        let stands = Isometry::translation(Vector::new(at[0], at[1], -2.0 * scale)).ok()?;
        let placed = match family {
            Family::Leaning => {
                let axis = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 9.0)).ok()?;
                Isometry::rotation(&axis, 0.05 + draw.fraction() / 4.0)
                    .ok()?
                    .compose(&stands)
            }
            Family::Lattice | Family::Turned | Family::AtAPowerOfTwo => stands,
        };
        let bar = model.add_round_bar(radius, 12.0 * scale, &placed).ok()?;
        if let Family::Turned = family {
            let axis = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).ok()?;
            let turn = Isometry::rotation(&axis, 0.1 + draw.fraction()).ok()?;
            model.place_solid(block, &turn);
            model.place_solid(bar, &turn);
        }
        Some((model, [block, bar]))
    }

    /// Cut every face of a drawn pair and say what each attempt came to.
    fn how_each_face_went(family: Family) -> (usize, BTreeMap<String, usize>) {
        let policy = Policy::certified();
        let mut draw = Draw(0x5EED);
        let mut faces = 0;
        let mut gave_up: BTreeMap<String, usize> = BTreeMap::new();
        for _ in 0..DRAWS {
            let Some((model, [block, bar])) = a_block_and_a_bar(family, &mut draw) else {
                continue;
            };
            let mut log = ProvenanceLog::new();
            let Ok(mut solids) = bake(&model, &[block, bar], &policy, &mut log) else {
                *gave_up.entry("baking".to_owned()).or_default() += 1;
                continue;
            };
            if cuts::gather(&mut solids, &policy, &mut log).is_err() {
                *gave_up.entry("gathering".to_owned()).or_default() += 1;
                continue;
            }
            for solid in &solids {
                for face in &solid.faces {
                    faces += 1;
                    match split_a_face(face, &policy, &mut log) {
                        Ok(_) => (),
                        Err(Error::Undecidable { records }) => {
                            let named = records
                                .records()
                                .last()
                                .map_or("nothing named", |record| record.predicate());
                            *gave_up.entry(named.to_owned()).or_default() += 1;
                        }
                        Err(why) => *gave_up.entry(format!("{why}")).or_default() += 1,
                    }
                }
            }
        }
        (faces, gave_up)
    }

    /// §31.7 criterion 16's control, for the half this lane owns: a curved pair
    /// written on the lattice is cut with nothing refused and nothing recorded.
    ///
    /// The control is what makes the headline mean anything (§17.7). A filter
    /// that refused everything would pass every property in this file and fail
    /// only here.
    #[test]
    fn every_certified_cut_of_a_lattice_curved_pair_is_decided() {
        let (faces, gave_up) = how_each_face_went(Family::Lattice);
        assert!(faces > 2000, "only {faces} faces were drawn");
        assert!(
            gave_up.is_empty(),
            "a lattice-aligned curved pair should refuse nothing, and this one refused {gave_up:?}"
        );
    }

    /// The other three families, measured and printed rather than judged against
    /// a number this file made up.
    ///
    /// §31.7 says no threshold is offered at a gate and the lane measures it, so
    /// the numbers are in §36 and what stands here is the shape of the answer:
    /// every refusal is named, and none of them is a decision anybody took
    /// without proof.
    #[test]
    fn how_often_a_certified_cut_of_a_turned_curved_pair_refuses() {
        for family in [Family::Turned, Family::Leaning, Family::AtAPowerOfTwo] {
            let (faces, gave_up) = how_each_face_went(family);
            let refused: usize = gave_up.values().sum();
            println!(
                "{}: {:.2}% of {faces} faces refused {gave_up:?}",
                family.named(),
                100.0 * refused as f64 / faces as f64
            );
            assert!(
                faces > 1000,
                "only {faces} faces were drawn for this family"
            );
            assert!(
                refused as f64 <= 0.30 * faces as f64,
                "{}: {refused} of {faces} is more than the three tenths §36 records",
                family.named()
            );
        }
    }
}

#[cfg(test)]
mod naming_a_surface {
    use rcad_geom::{Axis, Cylinder, Plane, Point, Sphere, Surface, Vector};

    use super::{
        in_the_order_a_float_compares, name_of_surface_bits, three_planes_faced_the_one_way,
    };

    /// The key a corner is named by has to order surfaces the way `rcad-geom`
    /// does, or a `which` counted along that crate's order sits beside a triple
    /// sorted along this one and two corners are named as one.
    ///
    /// `rcad-geom` compares the numbers a surface stores with
    /// [`f64::total_cmp`], so the key has to agree with that number for number.
    /// **Raw bits do not**: every negative `f64` has its sign bit set and sorts
    /// above every positive one, so the plane `z = 1` and the plane `z = -1`
    /// would come out the wrong way round — and there is no more ordinary a pair
    /// in a kernel than that.
    #[test]
    fn the_key_orders_numbers_the_way_they_compare() {
        let numbers = [
            f64::NEG_INFINITY,
            -1e300,
            -1.0,
            -f64::MIN_POSITIVE,
            -0.0,
            0.0,
            f64::MIN_POSITIVE,
            1.0,
            1e300,
            f64::INFINITY,
        ];
        for (index, &one) in numbers.iter().enumerate() {
            for &other in &numbers[index + 1..] {
                assert!(
                    in_the_order_a_float_compares(one) <= in_the_order_a_float_compares(other),
                    "{one} sorts above {other} as bits and below it as a number"
                );
            }
        }
        assert!(
            in_the_order_a_float_compares(-0.0) < in_the_order_a_float_compares(0.0),
            "total_cmp puts negative zero below positive zero and so does this"
        );
    }

    /// Planes before balls before pipes, which is what puts the two flat ones of
    /// a triple at the front.
    #[test]
    fn a_plane_sorts_before_a_ball_and_a_ball_before_a_pipe() {
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a radius"));
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a radius"));
        assert!(name_of_surface_bits(&flat) < name_of_surface_bits(&ball));
        assert!(name_of_surface_bits(&ball) < name_of_surface_bits(&pipe));
    }

    /// A triple asked for with one of its planes turned round is the same three
    /// planes handed to the solve, so it meets at the same point down to the
    /// last bit.
    ///
    /// Which is what a fuse needs: the two solids either side of a shared plane
    /// hold it facing opposite ways, and the answer must not depend on which of
    /// them the arrangement asked about the corner.
    #[test]
    fn a_plane_turned_round_is_handed_to_the_solve_the_same_way() {
        let at = |normal, through| {
            Plane::from_point_and_normal(through, normal).expect("a normal with length")
        };
        let wall = at(Vector::new(1.0, 2.0, 3.0), Point::new(3.0, 0.0, 0.0));
        let floor = at(Vector::new(0.0, 0.0, 1.0), Point::origin());
        let side = at(Vector::new(0.0, 1.0, 0.0), Point::origin());

        assert_eq!(
            three_planes_faced_the_one_way([wall, floor, side].map(Surface::Plane)),
            three_planes_faced_the_one_way([wall.reversed(), floor, side].map(Surface::Plane))
        );
    }

    /// A triple with a ball in it is handed over untouched, because turning one
    /// of its planes round would renumber `which` of the two meeting points a
    /// corner is.
    #[test]
    fn a_triple_with_a_ball_in_it_keeps_the_planes_it_was_given() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let floor = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("a normal with length");
        let side = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
            .expect("a normal with length");
        let asked = [Surface::Plane(floor), Surface::Plane(side), ball];

        assert_eq!(three_planes_faced_the_one_way(asked), asked);
    }
}
