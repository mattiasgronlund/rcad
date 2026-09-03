//! How often a certified fuse with a cone in it gives up, family by family.
//!
//! §17.7's discipline, unchanged: the rates are **never blended**, every family
//! prints its own number with its own denominator, and every refusal is named
//! rather than counted. What is different here is that a cone is the first
//! surface this kernel holds whose crossings it mostly cannot name, so the
//! headline is expected to be bad — and a bad rate reported is worth more than a
//! good rate got by only drawing the shapes that work.
//!
//! # What decides every number below
//!
//! `intersect` names a plane against a cone in exactly two of the seven shapes
//! they can meet in — the ellipse and the apex touch — and refuses the other five
//! by name. So the whole of a cone's fusibility today comes down to **which
//! planes a solid brings**:
//!
//! - a plane **square to the cone's axis** cuts a circle, which is named;
//! - a plane leaning less than the cone's own side cuts an ellipse, named;
//! - a plane **parallel to the axis**, which is four of a block's six faces when
//!   the block stands upright, cuts a **hyperbola**, and there is no arm in
//!   `EdgeRun` for one;
//! - anything curved against a cone is a degree-four space curve and is refused
//!   outright.
//!
//! The families are chosen to measure each of those rather than to average them
//! away.
//!
//! # **Every plane of the tool is asked, and that is a decision and not an
//! oversight**
//!
//! There is no face-level box test in the fuse, on purpose: `fuse/cuts.rs`'s own
//! note argues that skipping a face pair whose boxes miss *changes an answer*,
//! because a plane reaches past the face carrying it and two neighbours split at
//! the same point of the edge between them or the answer is not a manifold.
//! §17.4 threw the BVH out for the same reason arriving from the other side.
//!
//! So **a tool solid cannot hide its awkward planes by keeping them out of the
//! way**. A block standing upright beside a cone still has four planes parallel
//! to the cone's axis, every one of them is put to `intersect`, and every one of
//! them comes back a hyperbola. The consequence is worth stating plainly because
//! it is the whole of what a cone costs a fuse today:
//!
//! > A cone can be fused with a solid **only when every plane of that solid
//! > leans more steeply than the cone's own half-angle.**
//!
//! A cube stood on its corner is the tidiest solid that does: its three edge
//! directions are then equally inclined to the vertical, so all six of its
//! planes lean at the same angle, and `a_cube_on_its_corner_leans_every_one_of
//! _its_planes_across_a_shallow_cone` measures that angle rather than assuming
//! it.

use std::collections::BTreeMap;

use rcad_geom::{Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// How many pairs each family draws.
const DRAWS: usize = 16;

/// A deterministic sequence, so that a rate is one number and not a range.
struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    /// A whole number between `low` and `high`, both ends included.
    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }
}

/// The families, and what each is a measurement of.
#[derive(Clone, Copy)]
enum Family {
    /// A frustum against a cube stood on its corner, so that every one of the
    /// cube's six planes leans more steeply than the cone's own side and cuts
    /// it in an ellipse. **The control that says a cone can be fused at all.**
    AFrustumAgainstACornerUpCube,
    /// The same over a cone that ends in a point, so the answer carries the
    /// apex.
    APointedConeAgainstACornerUpCube,
    /// A frustum against a block that overlaps it sideways. Four of the block's
    /// faces run parallel to the cone's axis and cut it in hyperbolas.
    AFrustumAgainstABlock,
    /// A frustum against a round bar: cone against pipe, a degree-four space
    /// curve.
    AFrustumAgainstABar,
    /// Two coaxial frustums of different slopes: cone against cone.
    TwoFrustumsOnOneAxis,
}

impl Family {
    fn named(self) -> &'static str {
        match self {
            Family::AFrustumAgainstACornerUpCube => "a frustum against a corner-up cube",
            Family::APointedConeAgainstACornerUpCube => "a pointed cone against a corner-up cube",
            Family::AFrustumAgainstABlock => "a frustum against a block",
            Family::AFrustumAgainstABar => "a frustum against a bar",
            Family::TwoFrustumsOnOneAxis => "two frustums on one axis",
        }
    }
}

/// The turn that stands a cube on its corner: the diagonal `(1, 1, 1)` brought
/// upright.
///
/// Every one of the cube's six planes then leans at the same angle to the
/// vertical, which is the most a cube's planes can be made to lean *all at once*
/// — the three edge directions are orthonormal, so their vertical components
/// square to one and the smallest of them is never above `1/√3`.
fn stood_on_its_corner() -> Isometry {
    let axis = rcad_geom::Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0))
        .expect("a direction with length");
    Isometry::rotation(&axis, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn")
}

/// A box of the given lengths with its low corner at `at`.
fn a_block(
    model: &mut Model,
    [along_x, along_y, along_z]: [f64; 3],
    at: Vector,
) -> Option<SolidKey> {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .ok()?;
    model
        .add_extrusion(&rectangle, along_z, &Isometry::translation(at).ok()?)
        .ok()
}

fn a_pair(family: Family, draw: &mut Draw) -> Option<(Model, SolidKey, SolidKey)> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    // A power-of-two slope, so every corner of the cone sits exactly on it, and
    // never above `1/√2` so that a cube stood on its corner leans every one of
    // its planes across it — see the note at the top of the file.
    let slope = 2.0_f64.powi(i32::try_from(draw.next() % 3).ok()? - 3);
    let far = draw.whole(6, 12);
    let near = draw.whole(1, 4);

    Some(match family {
        Family::AFrustumAgainstACornerUpCube | Family::APointedConeAgainstACornerUpCube => {
            let cone = if matches!(family, Family::AFrustumAgainstACornerUpCube) {
                a_frustum(&mut model, slope, near, far)
            } else {
                a_pointed_cone(&mut model, slope, far)
            };
            let side = draw.whole(4, 9);
            let cube = a_block(&mut model, [side; 3], Vector::new(0.0, 0.0, 0.0))?;
            let up = draw.whole(1, far as i64 - 1);
            model.place_solid(
                cube,
                &Isometry::translation(Vector::new(0.0, 0.0, up))
                    .ok()?
                    .compose(&stood_on_its_corner()),
            );
            (model, cone, cube)
        }
        Family::AFrustumAgainstABlock => {
            let cone = a_frustum(&mut model, slope, near, far);
            let side = slope * far;
            // Clear of the apex on purpose: a block whose walls passed through
            // it would cut the cone in *two lines through the apex* and measure
            // that row instead of the hyperbola row, which is what this family
            // is for.
            let block = a_block(
                &mut model,
                [side, side, far],
                Vector::new(1.0, 1.0, near + 1.0),
            )?;
            (model, cone, block)
        }
        Family::AFrustumAgainstABar => {
            let cone = a_frustum(&mut model, slope, near, far);
            let bar = model
                .add_round_bar(
                    slope * near,
                    2.0 * far,
                    &Isometry::translation(Vector::new(0.0, 0.0, -far)).ok()?,
                )
                .ok()?;
            (model, cone, bar)
        }
        Family::TwoFrustumsOnOneAxis => {
            let cone = a_frustum(&mut model, slope, near, far);
            let other = a_frustum(&mut model, slope * 2.0, near, far);
            (model, cone, other)
        }
    })
}

/// Every rule, both ways round, over one family.
fn how_each_fuse_went(family: Family) -> (usize, usize, BTreeMap<String, usize>) {
    let policy = Policy::certified();
    let mut draw = Draw(0x5EED);
    let mut fuses = 0;
    let mut answered = 0;
    let mut gave_up: BTreeMap<String, usize> = BTreeMap::new();
    for _ in 0..DRAWS {
        let Some((mut model, cone, tool)) = a_pair(family, &mut draw) else {
            continue;
        };
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for (arguments, tools) in [([cone], [tool]), ([tool], [cone])] {
                fuses += 1;
                let mut log = ProvenanceLog::new();
                match model.fuse(&arguments, &tools, rule, &policy, &mut log) {
                    Ok(_) => {
                        answered += 1;
                        // A fuse hands back a sound model or it refuses. This is
                        // the half that makes the rate mean anything: a family
                        // that answered by writing a broken solid would look
                        // like a good rate.
                        model
                            .validate(&policy, &mut ProvenanceLog::new())
                            .expect("a fuse hands back a sound model or refuses");
                    }
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
    (fuses, answered, gave_up)
}

/// The measurement, printed family by family and never blended.
///
/// # What is asserted and what is only reported
///
/// The two curved-tool families are asserted at **nothing answered**: their
/// crossings are shapes this vocabulary has no arm for, and a fuse that answered
/// one would be answering something it cannot name — the wrong answer §4.1 ranks
/// below refusing.
///
/// The two corner-up-cube families are asserted at **something answered and
/// something refused**, and no more than that. Every plane of such a cube leans
/// more steeply than a shallow cone's own side, so `intersect` names every
/// crossing — but naming a crossing is where a cone's support currently ends,
/// and the arrangement, the selection and the mesher each have their own reasons
/// to give up further down. Those refusals are **named, counted and printed**,
/// and what they say is where the next lane's work is. A floor rather than a
/// ceiling is the honest assertion for a family whose real rate is the thing
/// being reported; a floor on the refusals too, because a family measured at
/// none refused would be a rate to re-derive rather than one to relax (§19.1).
///
/// The middle family, a frustum against an ordinary upright block, is asserted
/// at **nothing answered**: what it measures is what the hyperbola row costs a
/// real fuse, and every one of its fuses pays it today. It is pinned rather than
/// printed because the number that says a cone can be cut is this one, and a
/// number that only ever moves inside a `println!` is a number CI cannot tell
/// you about. **The assertion is meant to be broken**, by the lane that teaches
/// `where_a_plane_and_a_cone_cross` the hyperbola — and breaking it is the
/// evidence that lane landed.
///
/// **And the corner-up rate is not the cone's rate.** A round bar of the same
/// size in the same place, against the same cubes, answers exactly as often —
/// `a_frustum_and_a_bar_of_the_same_size_refuse_the_same_corner_up_cubes_alike`
/// is that control, and without it these numbers read as a cone's fault when
/// they are the family's.
#[test]
fn how_often_a_certified_fuse_with_a_cone_in_it_refuses() {
    let mut every = Vec::new();
    for family in [
        Family::AFrustumAgainstACornerUpCube,
        Family::APointedConeAgainstACornerUpCube,
        Family::AFrustumAgainstABlock,
        Family::AFrustumAgainstABar,
        Family::TwoFrustumsOnOneAxis,
    ] {
        let (fuses, answered, gave_up) = how_each_fuse_went(family);
        let refused: usize = gave_up.values().sum();
        println!(
            "{}: {fuses} fuses, {answered} answered, {refused} refused ({:.1}%): {gave_up:?}",
            family.named(),
            100.0 * refused as f64 / fuses as f64
        );
        assert!(fuses >= 90, "{}: only {fuses} fuses", family.named());
        every.push((family, fuses, answered, refused));
    }

    for (family, fuses, answered, refused) in every {
        match family {
            Family::AFrustumAgainstACornerUpCube | Family::APointedConeAgainstACornerUpCube => {
                assert!(
                    answered > 0,
                    "{}: every plane of a corner-up cube cuts a shallow cone in an ellipse, so \
                     some of these have to come through — and none of {fuses} did",
                    family.named()
                );
                assert!(
                    refused > 0,
                    "{}: measured at none refused of {fuses}, which is not what this family has \
                     ever given — re-derive the number rather than relax it",
                    family.named()
                );
            }
            Family::AFrustumAgainstABar | Family::TwoFrustumsOnOneAxis => assert_eq!(
                answered,
                0,
                "{}: a cone against another curved surface is a degree-four space curve nothing \
                 here can name, and this answered {answered} of {fuses}",
                family.named()
            ),
            Family::AFrustumAgainstABlock => assert_eq!(
                answered,
                0,
                "{}: every plane of an upright block that cuts this frustum at all cuts it in a \
                 hyperbola, which `where_a_plane_and_a_cone_cross` refuses, so none of {fuses} \
                 can come through — and {answered} did. If a lane just taught it the hyperbola, \
                 this is the test that says so: re-derive the row here rather than delete it",
                family.named()
            ),
        }
    }
}

/// A cube stood on its corner leans every one of its planes across a shallow
/// cone, and this is that angle measured rather than assumed.
///
/// Every family that fuses anything rests on it. What is checked is the sign the
/// intersector actually reads, `E = (n·w)² − slope²‖n × w‖²`, through
/// `intersect` itself rather than through a re-derivation of it: each of the
/// cube's six planes crossed with the cone has to come back one closed curve.
#[test]
fn a_cube_on_its_corner_leans_every_one_of_its_planes_across_a_shallow_cone() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = a_block(&mut model, [6.0; 3], Vector::new(0.0, 0.0, 0.0)).expect("a cube");
    model.place_solid(
        cube,
        &Isometry::translation(Vector::new(0.0, 0.0, 5.0))
            .expect("a shift")
            .compose(&stood_on_its_corner()),
    );

    let cone = rcad_geom::Surface::from(
        rcad_geom::Cone::new(rcad_geom::Axis::z(), 0.5).expect("a positive slope"),
    );
    let mut leans = Vec::new();
    for face in model.solid(cube).faces() {
        let flat = face
            .surface()
            .as_plane()
            .copied()
            .expect("every face of a cube is flat");
        leans.push(flat.normal().z().abs());
        let met = rcad_geom::intersect(
            &rcad_geom::Surface::from(flat),
            &cone,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane leaning across a cone cuts an ellipse");
        assert!(
            matches!(met, rcad_geom::SurfaceMeet::Curves(ref runs) if runs.len() == 1),
            "this plane cuts the cone in {met:?} rather than in one closed curve"
        );
    }
    let shallowest = leans.iter().copied().fold(f64::INFINITY, f64::min);
    println!("a corner-up cube's planes lean at least {shallowest} of the way to its axis");
    assert!(
        shallowest > 0.5 / 1.25f64.sqrt(),
        "every plane has to lean more steeply than the cone's own side, and the shallowest \
         leans {shallowest}"
    );
}

/// A frustum intersected with a cube that swallows it whole is the frustum, and
/// it still measures its closed form.
///
/// **The closed form through a real fuse.** Every one of the cube's six planes is
/// put to `intersect` against the cone and comes back an ellipse; none of those
/// ellipses lands on the face, so what comes out is the frustum again — and its
/// volume is `πh(R² + Rr + r²)/3` and its skin `π(R + r)·slant` plus two discs,
/// checked against the kernel's own published bound rather than against a number
/// recorded from a previous run.
#[test]
fn a_frustum_swallowed_by_a_cube_comes_back_the_same_frustum() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_frustum(&mut model, 0.5, 4.0, 8.0);
    let cube = a_block(&mut model, [40.0; 3], Vector::new(-20.0, -20.0, -20.0))
        .expect("a cube big enough to swallow the frustum");
    model.place_solid(cube, &stood_on_its_corner());

    let answer = model
        .fuse(&[cone], &[cube], Rule::Intersection, &policy, &mut log)
        .expect("every plane of a corner-up cube cuts a shallow cone in an ellipse");
    let pieces: Vec<SolidKey> = answer.solids().to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("and the answer is a sound model");
    assert_eq!(pieces.len(), 1, "a frustum inside a cube is one solid");

    let how_fine = rcad_geom::Deflection::of(0.002).expect("a positive length");
    let measured = model
        .solid(pieces[0])
        .measure(how_fine, &policy, &mut ProvenanceLog::new())
        .expect("the answer measures");

    let (small, big, height) = (2.0f64, 4.0f64, 4.0f64);
    let volume = core::f64::consts::PI * height * (big * big + big * small + small * small) / 3.0;
    let slant = (height * height + (big - small) * (big - small)).sqrt();
    let area = core::f64::consts::PI * ((big + small) * slant + big * big + small * small);
    println!(
        "a frustum through an intersection: volume {} against {volume}, area {} against {area}",
        measured.volume(),
        measured.surface_area()
    );
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the answer's volume {} misses πh(R² + Rr + r²)/3 = {volume} by more than the bound {}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.surface_area() - area).abs() <= measured.area_error(),
        "the answer's area {} misses {area} by more than the bound {}",
        measured.surface_area(),
        measured.area_error()
    );
}

/// A cone-faced solid taken apart through views alone and described again is the
/// same solid.
///
/// The round trip §42.12 criterion 1 asks for, over a cone. Nothing is read out
/// of the arena: the faces, wires, coedges, edges and corners all come through
/// `FaceView` and friends, and what goes back in is a fresh `SolidBuilder`
/// description. The two are then required to measure the same **bit for bit**
/// and to answer the same fuse against a third solid — a corner-up cube taking a
/// bite out of each, with every piece's volume, area and centroid compared and
/// the piece count compared.
#[test]
fn a_frustum_taken_apart_through_views_and_described_again_is_the_same_frustum() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let built = a_frustum(&mut model, 0.5, 4.0, 8.0);

    let rebuilt = cones::describe_again(&model, built)
        .expect("a frustum can be described again")
        .build(&mut model, &policy, &mut log)
        .expect("and the sweep is happy with the description");
    model
        .validate(&policy, &mut log)
        .expect("the model holding both is sound");

    let how_fine = rcad_geom::Deflection::of(0.01).expect("a positive length");
    let [one, other] = [built, rebuilt].map(|solid| {
        model
            .solid(solid)
            .measure(how_fine, &policy, &mut ProvenanceLog::new())
            .expect("both measure")
    });
    assert_eq!(one.volume(), other.volume());
    assert_eq!(one.surface_area(), other.surface_area());
    assert_eq!(one.centroid(), other.centroid());

    // And the same cut, on a clone of the model each time, has to give the same
    // pieces — which is what says the two descriptions are the same *solid* and
    // not merely two solids of the same size.
    let cut = |which: SolidKey| {
        let mut copy = model.clone();
        let cube = a_block(&mut copy, [5.0; 3], Vector::new(0.0, 0.0, 0.0)).expect("a cube");
        copy.place_solid(cube, &stood_on_its_corner());
        let answer = copy
            .fuse(
                &[which],
                &[cube],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a corner-up cube cuts a shallow cone in ellipses");
        let keys: Vec<SolidKey> = answer.solids().to_vec();
        assert!(!keys.is_empty(), "the cut has to leave something behind");
        let mut pieces: Vec<(f64, f64, [f64; 3])> = keys
            .iter()
            .map(|&piece| {
                let measured = copy
                    .solid(piece)
                    .measure(how_fine, &policy, &mut ProvenanceLog::new())
                    .expect("each piece measures");
                (
                    measured.volume(),
                    measured.surface_area(),
                    measured.centroid().coordinates(),
                )
            })
            .collect();
        pieces.sort_by(|one, other| one.0.total_cmp(&other.0));
        pieces
    };
    let from_the_original = cut(built);
    let took_material_away =
        from_the_original.iter().map(|piece| piece.0).sum::<f64>() < one.volume() - 0.01;
    assert!(
        took_material_away,
        "the cut has to take material away, or this compared nothing: {from_the_original:?}"
    );
    assert_eq!(from_the_original, cut(rebuilt));
}

/// A point inside a cone-faced solid is inside it, and one outside is outside.
///
/// The ray caster's cone arm, which is the one place a cone's quadratic can have
/// a **negative** squared term — every ray pointing further into the cone than
/// its own rulings do. A ball's and a pipe's never can, and the code said so in
/// an `unreachable!` until this lane.
#[test]
fn a_grid_of_points_is_classified_the_way_the_cone_says() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_frustum(&mut model, 0.5, 4.0, 8.0);
    let view = model.solid(solid);

    let inside = |at: Point| {
        let height = at.z();
        let across = at.x().hypot(at.y());
        height > 4.0 && height < 8.0 && across < 0.5 * height
    };
    let outside = |at: Point| {
        let height = at.z();
        let across = at.x().hypot(at.y());
        !(4.0..=8.0).contains(&height) || across > 0.5 * height
    };

    let mut counted = 0;
    for x in -6..=6 {
        for y in -6..=6 {
            for z in 0..=12 {
                let at = Point::new(f64::from(x) * 0.7, f64::from(y) * 0.7, f64::from(z) * 0.9);
                let held = view.classify_point(at, &policy, &mut ProvenanceLog::new());
                if inside(at) {
                    assert_eq!(held, Containment::Inside, "{at:?} is inside the frustum");
                    counted += 1;
                } else if outside(at) {
                    assert_eq!(held, Containment::Outside, "{at:?} is outside the frustum");
                    counted += 1;
                }
            }
        }
    }
    assert!(counted > 1000, "only {counted} points were decided");
}

/// **The control that says the rate above is the family's and not the cone's.**
///
/// A frustum against a corner-up cube answers 9 of 96, which reads like a
/// damning number for a cone until the same cubes are put to a **round bar** of
/// the same size in the same place. It answers **9 of 96 too**, and refuses in
/// the same two ways in nearly the same proportions:
///
/// | | answered | wire will not lie flat | a decision nobody could settle |
/// |---|---:|---:|---:|
/// | a frustum | 9 | 27 | 60 |
/// | a round bar of the same size | 9 | 21 | 66 |
///
/// So what those 87 refusals measure is **a hard family**, not a hard surface: a
/// cube stood on its corner brings six planes at an awkward angle to a curved
/// solid, and the arrangement gives up at about the same rate on a cylinder,
/// which has been in this kernel since M2. A cone's own cost is the row above
/// it in `how_often_a_certified_fuse_with_a_cone_in_it_refuses` — the upright
/// block, refused 96 of 96 for a hyperbola — and that one a bar answers
/// completely.
///
/// Without this control the cone's number would have been read as the cone's
/// fault and the next lane would have gone looking in the wrong place.
///
/// **What is checked is a floor, not a band.** The bar is the floor the frustum
/// has to stand on; the frustum answering *more* is not a failure of the claim
/// but this kernel learning a crossing with a cone on it. The bar has no
/// hyperbola row to gain, so it cannot follow that gain, and a band on both
/// sides would report a cone as the family's problem exactly when the cone had
/// stopped being one.
#[test]
fn a_frustum_and_a_bar_of_the_same_size_refuse_the_same_corner_up_cubes_alike() {
    let policy = Policy::certified();
    let mut gave_up: [BTreeMap<String, usize>; 2] = [BTreeMap::new(), BTreeMap::new()];
    let mut answered = [0usize; 2];
    let mut tries = 0;
    let mut draw = Draw(0x5EED);

    for _ in 0..DRAWS {
        let slope = 2.0_f64.powi(i32::try_from(draw.next() % 3).expect("a small number") - 3);
        let far = draw.whole(6, 12);
        let near = draw.whole(1, 4);
        let side = draw.whole(4, 9);
        let up = draw.whole(1, far as i64 - 1);

        for which in 0..2 {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            // A bar as wide as the frustum's wide end and as tall as it is, so
            // the cube meets the same amount of curved surface either way.
            let solid = if which == 0 {
                a_frustum(&mut model, slope, near, far)
            } else {
                model
                    .add_round_bar(
                        slope * far,
                        far - near,
                        &Isometry::translation(Vector::new(0.0, 0.0, near))
                            .expect("a finite shift"),
                    )
                    .expect("a bar of that radius")
            };
            let Some(cube) = a_block(&mut model, [side; 3], Vector::new(0.0, 0.0, 0.0)) else {
                continue;
            };
            model.place_solid(
                cube,
                &Isometry::translation(Vector::new(0.0, 0.0, up))
                    .expect("a finite shift")
                    .compose(&stood_on_its_corner()),
            );
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                for (arguments, tools) in [([solid], [cube]), ([cube], [solid])] {
                    if which == 0 {
                        tries += 1;
                    }
                    match model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new()) {
                        Ok(_) => answered[which] += 1,
                        Err(why) => *gave_up[which].entry(format!("{why}")).or_default() += 1,
                    }
                }
            }
        }
    }

    println!("over {tries} fuses each:");
    println!("  a frustum: {} answered, {:?}", answered[0], gave_up[0]);
    println!(
        "  a bar of the same size: {} answered, {:?}",
        answered[1], gave_up[1]
    );
    assert!(tries >= 90, "only {tries} fuses were drawn");
    // **A floor and not a band.** The bar is a floor on the frustum: a cone is
    // not what makes this family hard, so the frustum must not answer *fewer*
    // than the bar by more than a quarter. It may answer more, and a two-sided
    // band would go red saying the opposite of what had happened — the bar has
    // no hyperbola row to gain, so every crossing a cone learns to name is a
    // gain the control cannot follow.
    let shortfall = answered[1] as f64 - answered[0] as f64;
    assert!(
        shortfall <= 0.25 * (answered[1] as f64).max(4.0),
        "a bar answered {} of these and a frustum only {}, so a cone *is* what makes this \
         family hard, and the note above is wrong",
        answered[1],
        answered[0]
    );
    assert!(
        answered[1] > 0 && answered[1] < tries,
        "the bar has to answer some and refuse some, or it is not a control"
    );
}

/// `A ∪ B` is `B ∪ A` and `A ∩ B` is `B ∩ A`, over every cone pair that answers.
///
/// The metamorphic half of `fuses_that_must_agree.rs`, over a surface kind that
/// did not exist when that file was written. Union and intersection are the two
/// rules that commute; difference does not and is left out rather than fudged.
///
/// **Compared on the answer's own geometry and not on a measurement of it**, and
/// the first attempt at this compared measurements and failed by `5e-14` on a
/// volume of 220. That was not the fuse disagreeing: the two answers are the
/// same solid, and what differed was where each mesh happened to start round the
/// cone. A measurement is a mesh away from the thing being compared, so the
/// comparison is on what the fuse actually wrote — every corner of every face,
/// bit for bit.
///
/// **The count that answered is printed and floored**, because a family that
/// refused every fuse would satisfy this vacuously — §14.7's rule, and the whole
/// reason the two rates above are reported rather than assumed.
#[test]
fn uniting_a_cone_with_a_cube_gives_the_same_answer_whichever_way_round_it_is_asked() {
    let policy = Policy::certified();
    let mut compared = 0;
    let mut both_refused = 0;

    for family in [
        Family::AFrustumAgainstACornerUpCube,
        Family::APointedConeAgainstACornerUpCube,
    ] {
        let mut draw = Draw(0x5EED);
        for _ in 0..DRAWS {
            let Some((model, cone, cube)) = a_pair(family, &mut draw) else {
                continue;
            };
            for rule in [Rule::Union, Rule::Intersection] {
                let written = |arguments: [SolidKey; 1], tools: [SolidKey; 1]| {
                    let mut copy = model.clone();
                    let answer =
                        copy.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())?;
                    let keys: Vec<SolidKey> = answer.solids().to_vec();
                    copy.validate(&policy, &mut ProvenanceLog::new())?;
                    let mut corners: Vec<[u64; 3]> = Vec::new();
                    for piece in keys {
                        for face in copy.solid(piece).faces() {
                            for wire in face.wires() {
                                for at in wire.points() {
                                    corners.push(at.coordinates().map(f64::to_bits));
                                }
                            }
                        }
                    }
                    corners.sort_unstable();
                    Ok::<_, rcad_topo::Error>(corners)
                };
                match (written([cone], [cube]), written([cube], [cone])) {
                    (Ok(one), Ok(other)) => {
                        assert!(
                            !one.is_empty(),
                            "{rule:?} of a cone and a cube came back with no corners at all"
                        );
                        assert_eq!(
                            one, other,
                            "{rule:?} of a cone and a cube wrote different corners depending on \
                             which was named first"
                        );
                        compared += 1;
                    }
                    (Err(_), Err(_)) => both_refused += 1,
                    // One way round answered and the other refused, which is
                    // allowed: a refusal is about the arithmetic and the two
                    // orders ask different questions of it. Nothing to compare.
                    _ => (),
                }
            }
        }
    }

    println!(
        "a cone and a corner-up cube, both ways round: {compared} pairs compared, \
         {both_refused} refused both ways"
    );
    assert!(
        compared >= 4,
        "only {compared} pairs were ever compared, so this proved almost nothing"
    );
}
