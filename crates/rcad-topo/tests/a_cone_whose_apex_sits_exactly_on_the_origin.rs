//! What a consumer can and cannot build out of a cone's apex when that apex
//! reads exactly `[0, 0, 0]`.
//!
//! # The shape of the problem
//!
//! A cone has no normal at its apex, so a consumer shading one has to find the
//! apex among the mesh points to know not to ask for a normal there
//! (`docs/decisions.md` §56.1). It has two ways to look:
//!
//! - **Exact equality** against [`Cone::apex`]. That works or it does not, and a
//!   miss is a confidently wrong normal rather than a refusal.
//! - **A ball around the apex**, so that a mesh point rcad's arithmetic moved a
//!   whisker off the apex is still recognised as the apex.
//!
//! The only length a consumer has to hand when it sizes that ball is the apex's
//! own coordinates, so the ball comes out as some multiple of
//! `f64::EPSILON · |apex|`. **When the apex sits on the origin that is exactly
//! zero, for every multiple**, and the ball degenerates to the exact test it was
//! meant to back up.
//!
//! # What this file demonstrates, in two halves
//!
//! [`the_mesh_lands_bit_for_bit_on_an_apex_that_reads_exactly_zero`] is the half
//! that is **true today and rests on nothing published**. The apex a cone face
//! reports and the point its mesh puts there are the same bits, so the exact
//! test succeeds and the degenerate ball costs nobody anything. §56 is why: the
//! mesh used to ride a placed apex out on its placement **inverted twice** and
//! land an ulp off, and a consumer reached that with a cone whose apex read
//! zero. That demonstration is gone because the defect it rode on was fixed, and
//! this test is what stops it coming back unremarked.
//!
//! [`the_floor_under_that_mesh_is_not_zero_however_close_the_apex_is_to_the_origin`]
//! is the half that says the first one is **luck a consumer cannot bank on**.
//! rcad publishes, through `Error::DeflectionBelowWhatAMeshCanReach`, how close
//! it can place a mesh point of a given solid at all — and that floor is
//! strictly positive, rises in step with how far the cone reaches, and has
//! nothing whatever to do with where the apex sits. A cone reaching a
//! thousand million with its apex on the origin is a solid rcad says it cannot
//! place a point on to better than a few hundredths, while every ball a consumer
//! can size off that apex has radius exactly zero.
//!
//! So the two numbers that ought to be the same quantity — how far the mesh may
//! have moved the apex, and how wide the ball has to be to catch it — differ by
//! the whole of the floor, unboundedly, and the only thing standing in the gap
//! is a bit-for-bit equality that appears in no doc comment a consumer can read.
//!
//! # Every route to an apex reading zero, and why the three of them are all
//!
//! A cone face either carries a placement or does not — that is literally the
//! branch in `the_frame_this_face_is_meshed_in` and in
//! `back_out_of_the_frame_this_face_is_meshed_in` — and its mesh either reaches
//! the apex or does not. That partitions every cone face this kernel can hold:
//!
//! 1. **Unplaced, reaching the apex.** The frame is
//!    `a_shift_that_loses_nothing`, which folds the apex into the run it takes
//!    its shift from; an apex on the origin makes every coordinate of that run
//!    straddle zero, so the shift is refused and the frame is the identity. This
//!    is what a hand-built solid, a fused one and an imported one all are, and
//!    [`the_sizes`] sweeps it up to a reach of `1e9`.
//! 2. **Placed, reaching the apex.** The apex reads zero only if the placement
//!    carries it to zero, and with a stored apex on the origin that means a
//!    translation of exactly zero — a pure turn about an axis through the
//!    origin. [`the_turns`] sweeps fifteen of them.
//! 3. **Not reaching the apex at all**, which is a frustum. There is no apex
//!    point to find and the exact test correctly finds none. That is the
//!    *other* end of the ball's problem — a rim standing on a real ruling being
//!    swallowed by a ball too wide — and not this one.
//!
//! Route 2 cannot reach the defect §56 fixed, and that is a demonstration rather
//! than a hope: a translation of exactly zero survives `Isometry::inverse` twice
//! because it is zero and stays zero.
//! `snapping_a_cones_apex_onto_the_origin.rs` counts that over its own 24
//! placements. What route 2 is here for is that it is the one a placement
//! *could* have rounded, so a sweep that only ever meshed at the identity would
//! be measuring the easy case and calling it the case.
//!
//! [`Cone::apex`]: rcad_geom::Cone::apex

mod cones;

use cones::a_pointed_cone;
use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, SolidKey};

/// A power of two, so `slope · height` is exact and the rim corner is certified
/// on its own cone — `cones/mod.rs` says why that matters.
const SLOPE: f64 = 0.5;

/// How far up its axis each cone's base cap sits. Nine decades, because the
/// floor's whole point is that it grows with this while the apex does not move.
///
/// **The apex is exactly zero on purpose and everything else here is not.** A
/// sweep whose other coordinates are all powers of two samples only the
/// zero-mantissa side of every rounding tie and can report a clean answer it has
/// not earned, which happened twice in this repository's neighbourhood on
/// 2026-09-06. So `1` and `8` have company: `1e3`, `1e6` and `1e9` are not
/// powers of two, and `2731.409382746502` fills its mantissa. The slope has to
/// stay a power of two — `cones/mod.rs` says why — and the rim corner is
/// `slope · far`, which is exact for every one of these.
const THE_SIZES: [f64; 6] = [1.0, 8.0, 1e3, 2731.409382746502, 1e6, 1e9];

/// The exact test a consumer makes, and the ball it would fall back on.
///
/// The mesh point nearest a cone's apex is the apex itself, bit for bit, at
/// every size and under every turn — so exact equality against
/// [`Cone::apex`](rcad_geom::Cone::apex) is enough and the degenerate ball is
/// never asked to do any work.
///
/// **What stops this passing for the wrong reason** is the second and third
/// assertions. A mesh with one point in it, or a cone whose whole rim collapsed
/// onto the apex, would satisfy the first one and mean nothing; so every row
/// also has to hold a real mesh, and the *next* point out has to be a long way
/// off — further than the floor by many decades, which is what says the
/// bit-for-bit hit is a hit and not a tie among points that are all at zero.
#[test]
fn the_mesh_lands_bit_for_bit_on_an_apex_that_reads_exactly_zero() {
    let mut rows = 0;
    for (label, placement) in the_turns() {
        for far in THE_SIZES {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let solid = a_pointed_cone(&mut model, SLOPE, far);
            model
                .place_solid(solid, &placement)
                .expect("this model made the key");

            let apex = the_apex_a_consumer_reads(&model, solid);
            assert_eq!(
                apex.coordinates(),
                [0.0, 0.0, 0.0],
                "{label}, reaching {far}: the apex is supposed to read exactly the origin, and \
                 this sweep says nothing about a cone whose apex does not"
            );

            let mut distances = how_far_each_mesh_point_is_from(&model, solid, apex, far / 800.0);
            distances.sort_by(f64::total_cmp);
            assert_eq!(
                distances[0], 0.0,
                "{label}, reaching {far}: the nearest mesh point to the apex is {} away, so a \
                 consumer that special-cases the apex on exact equality shades the tip with a \
                 radial normal instead of refusing",
                distances[0]
            );
            assert!(
                distances.len() > 50,
                "{label}, reaching {far}: only {} mesh points, which is not a cone",
                distances.len()
            );
            assert!(
                distances[1] > 1e6 * the_floor_under_the_mesh_of(&model, solid),
                "{label}, reaching {far}: the second nearest mesh point is only {} from the apex, \
                 against a floor of {}, so landing on the apex is a tie between points the mesher \
                 could not place apart rather than a hit",
                distances[1],
                the_floor_under_the_mesh_of(&model, solid)
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 96, "the sweep is supposed to be 16 turns by 6 sizes");
}

/// The half that says the test above is unfunded.
///
/// Three numbers per row: the widest ball a consumer can size off this apex,
/// which is zero because the apex is; the floor rcad publishes under this
/// solid's mesh, which is not; and what the second buys over the first, which is
/// all of it.
///
/// **The floor is charged against the cone's reach, not its apex.** The last
/// assertion is what says so rather than merely observing a positive number:
/// across nine decades of reach, with the apex nailed to the origin throughout,
/// the floor rises by nine decades too. A floor that was a constant, or one that
/// scaled off the apex, would go red there — and a ball sized off the apex is
/// exactly the second of those.
#[test]
fn the_floor_under_that_mesh_is_not_zero_however_close_the_apex_is_to_the_origin() {
    let mut floors = Vec::new();
    for far in THE_SIZES {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let solid = a_pointed_cone(&mut model, SLOPE, far);
        let apex = the_apex_a_consumer_reads(&model, solid);
        let floor = the_floor_under_the_mesh_of(&model, solid);

        for multiple in [1.0, 16.0, 1e6, 1e12] {
            assert_eq!(
                multiple * f64::EPSILON * apex.distance_to(Point::origin()),
                0.0,
                "reaching {far}: a ball of {multiple} epsilons of the apex's own length is \
                 supposed to be exactly zero wide, which is the whole problem"
            );
        }
        assert!(
            floor > 0.0,
            "reaching {far}: rcad publishes a floor of {floor} under this mesh, so there is \
             nothing here for the ball to be too small for"
        );
        println!("reaching {far:e}: ball 0, floor {floor:e}");
        floors.push(floor);
    }

    let (smallest, largest) = (floors[0], floors[floors.len() - 1]);
    let sizes = THE_SIZES[THE_SIZES.len() - 1] / THE_SIZES[0];
    assert!(
        (largest / smallest / sizes - 1.0).abs() < 1e-6,
        "the floor went from {smallest} to {largest} over a {sizes}-fold reach, so it is not \
         charged against the reach and this file's account of why the ball is the wrong shape \
         needs rewriting"
    );
}

/// §16.10's demonstration for route 2, and the positive control on the other
/// two tests.
///
/// [`the_mesh_lands_bit_for_bit_on_an_apex_that_reads_exactly_zero`] would look
/// exactly the same if the arithmetic it sweeps could not go wrong in the first
/// place, so this measures that the placements it sweeps are **the** placements
/// that keep an apex at zero, and that those are precisely the ones §56's defect
/// could not move.
///
/// The defect was riding a placed point out on its placement **inverted twice**,
/// and `Isometry::inverse` replaces the shift with `−R⁻¹t`, which turns a vector
/// and rounds. A turn about an axis through the origin has `t` exactly zero, so
/// there is nothing to round and nothing to lose. Put a shift on the same turn
/// and there is — which is the second half here, and it is what says this
/// measurement can fire at all rather than being blind.
///
/// **A shift is also what takes the apex off zero**, exactly and always: the
/// apex is stored at the origin, so the placed apex *is* the placement's whole
/// translation. So the two families do not merely differ, they are the same
/// split seen twice — "the apex still reads zero" and "the old route could not
/// have moved it" pick out the same placements.
#[test]
fn the_placements_that_keep_the_apex_at_zero_are_the_ones_the_old_route_could_not_move() {
    let mut kept_the_apex_at_zero = 0;
    let mut the_old_route_would_have_moved_it = 0;
    let mut worst = 0.0_f64;

    for (label, turn) in the_turns() {
        for shift in the_shifts() {
            let placement = shift.compose(&turn);
            let apex = placement.apply_to_point(Point::origin());
            let moved = placement
                .inverse()
                .inverse()
                .apply_to_point(Point::origin())
                .distance_to(apex);

            let stayed_at_zero = apex.coordinates() == [0.0, 0.0, 0.0];
            assert_eq!(
                stayed_at_zero,
                shift == Isometry::identity(),
                "{label}: an apex stored at the origin reads the placement's own translation, so \
                 it is at zero exactly when the placement does not shift"
            );
            if stayed_at_zero {
                assert_eq!(
                    moved, 0.0,
                    "{label}: this placement keeps the apex at zero, so the doubled inversion is \
                     supposed to be unable to move it, yet it moves it {moved}"
                );
                kept_the_apex_at_zero += 1;
            } else if moved > 0.0 {
                the_old_route_would_have_moved_it += 1;
                worst = worst.max(moved);
            }
        }
    }

    assert_eq!(
        kept_the_apex_at_zero, 16,
        "the sixteen placements the other tests sweep are supposed to be the whole of what keeps \
         an apex at zero"
    );
    assert!(
        the_old_route_would_have_moved_it > 20,
        "only {the_old_route_would_have_moved_it} shifted placements would have moved the apex on \
         the doubled inversion, so this measurement is not sharp enough to be a control on the \
         sixteen that stay put"
    );
    println!(
        "{kept_the_apex_at_zero} placements keep the apex at zero and none of them could have \
         moved it; {the_old_route_would_have_moved_it} shifted ones would have, worst {worst:e}"
    );
}

/// The identity and fifteen turns about axes through the origin, which are the
/// placements that leave an apex stored at the origin reading exactly zero.
fn the_turns() -> Vec<(String, Isometry)> {
    let mut turns = vec![("no placement at all".to_owned(), Isometry::identity())];
    for direction in [
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(1.0, 2.0, 3.0),
        Vector::new(-3.0, 1.0, 1.0),
        // Full mantissas in all three, for the reason [`THE_SIZES`] gives.
        Vector::new(
            0.427_183_059_624_719_3,
            -0.731_827_364_918_274_6,
            1.209_384_756_102_938_4,
        ),
    ] {
        let axis = Axis::new(Point::origin(), direction).expect("a non-zero direction");
        for angle in [0.37, 1.9, 2.8] {
            turns.push((
                format!("turned {angle} about {direction:?}"),
                Isometry::rotation(&axis, angle).expect("a turn about an axis through the origin"),
            ));
        }
    }
    turns
}

/// No shift, and three that are shifts, for the control above.
fn the_shifts() -> [Isometry; 4] {
    [
        Isometry::identity(),
        Isometry::translation(Vector::new(3.0, 5.0, 7.0)).expect("a shift"),
        Isometry::translation(Vector::new(-1.5, 0.25, 11.0)).expect("a shift"),
        Isometry::translation(Vector::new(
            0.0,
            2.930_471_528_364_917,
            1_000_000.483_920_175,
        ))
        .expect("a shift"),
    ]
}

fn the_apex_a_consumer_reads(model: &Model, solid: SolidKey) -> Point {
    let mut apexes = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| match face.surface() {
            Surface::Cone(cone) => Some(cone.apex()),
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Torus(_) => {
                None
            }
            other => panic!("a pointed cone has no {other:?} face"),
        });
    let apex = apexes.next().expect("a pointed cone has a cone face");
    assert!(apexes.next().is_none(), "and it has only the one");
    apex
}

fn how_far_each_mesh_point_is_from(
    model: &Model,
    solid: SolidKey,
    apex: Point,
    deflection: f64,
) -> Vec<f64> {
    model
        .solid(solid)
        .expect("this model made the key")
        .mesh(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a pointed cone meshes at a thousandth of its own height")
        .points()
        .iter()
        .map(|point| point.distance_to(apex))
        .collect()
}

/// How close rcad says it can place a mesh point of this solid at all.
///
/// It is published on the refusal path and nowhere else, so the way to read it
/// is to ask for a mesh nobody could cut and take the number out of the
/// complaint. That is `docs/decisions.md` §56.7's "half of demand 4".
fn the_floor_under_the_mesh_of(model: &Model, solid: SolidKey) -> f64 {
    let asked = Deflection::of(f64::MIN_POSITIVE).expect("the smallest positive length there is");
    match model.solid(solid).expect("this model made the key").mesh(
        asked,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Err(Error::DeflectionBelowWhatAMeshCanReach { closest, .. }) => closest,
        other => panic!("asking for a mesh no arithmetic could cut gave {other:?}"),
    }
}
