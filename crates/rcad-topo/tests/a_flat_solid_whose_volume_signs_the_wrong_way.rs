//! Flat solids whose volume is small beside their own size, and the sign the
//! divergence-theorem sum gives them.
//!
//! `docs/decisions.md` §49.8's class: a shape decided by a rounded comparison
//! against zero. The sum is `a·(b×c)` over the triangles a polyhedron's faces
//! fan into, accumulated in plain `f64`, and its sign says what the solid *is*
//! — [`SolidView::measure`] and `Model::validate` call a negative one
//! `Invalid::VolumeIsNotPositive`, and the fuse's `nest_the_shells` reads the
//! same sign to sort a shell into a lump or a void of one.
//!
//! **Neither family here is thin along an axis.** One power of two per axis is
//! taken out of the sum before it is added up, and that is exactly why an
//! axis-aligned sliver is safe: every term of a `3 × 3` determinant carries one
//! factor from each axis, so shrinking an axis shrinks every term by the same
//! exact amount and the terms of a flat-in-`z` solid are all as small as its
//! volume. A solid thin along a *diagonal* has all three axis extents at its
//! full size, every term is that size cubed, and the sum has to cancel down to
//! the volume. That cancellation is what these two families buy.
//!
//! **The oracle is not the kernel's own arithmetic.** For the prisms it is
//! [`orient2d`], which is exact: a profile that turns anticlockwise has a
//! positive area, and a prism of positive height over it encloses `area ×
//! height` of space. Nothing rounded is consulted to know the answer's sign.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, orient2d};
use rcad_topo::{Model, ProfileBuilder};

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// A triangle three ulps off its own long side, extruded one unit.
///
/// [`orient2d`] proves the three profile points turn anticlockwise, so the
/// triangle has area and the prism over it holds `area × 1` of space. The fan
/// says `-2.3e-18`.
#[test]
fn a_prism_over_a_triangle_that_provably_turns_anticlockwise_has_a_positive_volume() {
    let corners = [
        [0.5816321180784307, 0.2599163953511716],
        [0.057749314723031464, -0.5186392868204934],
        [0.4075509090987479, 0.0012098420106508303],
    ];
    assert_eq!(
        orient2d(corners[0], corners[1], corners[2]),
        Decision::Pos,
        "the exact predicate says this triangle turns anticlockwise, so it has area"
    );

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let profile = ProfileBuilder::new(corners.to_vec())
        .build(&policy, &mut log)
        .expect("a triangle that turns anticlockwise is a profile");
    let prism = model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("a prism one unit tall");

    let measured = model
        .solid(prism)
        .measure(Deflection::for_scale(model.scale()), &policy, &mut log)
        .expect("a prism over a triangle with area encloses space");
    assert!(
        measured.volume() > 0.0,
        "a prism over an anticlockwise triangle measured {}",
        measured.volume()
    );
}

/// How many prisms of that shape the fan gets the sign of wrong, over a family
/// rather than the one case above.
///
/// Two random points and a third rounded onto the line between them and then
/// nudged a few ulps off it. [`orient2d`] settles which way each one really
/// turns and the outline is written in that order, so every prism built here
/// encloses space. The seed is fixed, so the count is a number and not a mood.
#[test]
fn no_prism_over_a_triangle_with_area_is_refused_for_having_none() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut seed = Seed(12345);
    let mut built = 0;
    let mut refused = Vec::new();
    for _ in 0..20_000 {
        let one = [seed.between_minus_one_and_one(), seed.between_minus_one_and_one()];
        let other = [seed.between_minus_one_and_one(), seed.between_minus_one_and_one()];
        let along = seed.unit();
        let on_the_line = [
            one[0] + along * (other[0] - one[0]),
            one[1] + along * (other[1] - one[1]),
        ];
        let nudged = [
            on_the_line[0],
            ulps_away(on_the_line[1], i64::from(seed.next() as u32 % 5) - 2),
        ];
        let outline = match orient2d(one, other, nudged) {
            Decision::Pos => vec![one, other, nudged],
            Decision::Neg => vec![one, nudged, other],
            _ => continue,
        };
        let Ok(profile) = ProfileBuilder::new(outline).build(&policy, &mut log) else {
            continue;
        };
        let mut model = Model::new(a_working_size());
        let prism = model
            .add_extrusion(&profile, 1.0, &Isometry::identity())
            .expect("a prism one unit tall");
        built += 1;
        if let Err(complaint) =
            model
                .solid(prism)
                .measure(Deflection::for_scale(model.scale()), &policy, &mut log)
        {
            refused.push((one, other, nudged, format!("{complaint:?}")));
        }
    }
    assert!(built > 10_000, "only {built} of the draws became prisms");
    assert!(
        refused.is_empty(),
        "{} of {built} prisms with area were refused, the first being {:?}",
        refused.len(),
        refused.first()
    );
}

/// A rod down the `(1, 1, 1)` diagonal, three ten-thousandths of a millimetre
/// across, whose every corner is three planes meeting at right angles.
///
/// **The other half of the family, and the one that has no near-parallel
/// planes in it at all.** The prisms above are thin because two of their side
/// walls nearly agree, which is a shape the arrangement can refuse on its own
/// grounds. A square rod refuses nothing: its walls meet squarely, its corners
/// are as well conditioned as a cube's, and its volume is still `1e-18` beside
/// terms of size one because the rod does not run down an axis.
#[test]
fn a_square_rod_down_a_diagonal_holds_the_space_its_own_cross_section_says() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let across = 1e-9;
    let mut model = Model::new(a_working_size());
    let square = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [across, 0.0],
        [across, across],
        [0.0, across],
    ])
    .build(&policy, &mut log)
    .expect("a square");
    let leaning = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(-1.0, 1.0, 0.0)).expect("an axis with a length"),
        (1.0f64 / 3.0f64.sqrt()).acos(),
    )
    .expect("a real angle");
    let rod = model
        .add_extrusion(&square, 1.0, &leaning)
        .expect("a rod one unit long");

    let measured = model
        .solid(rod)
        .measure(Deflection::for_scale(model.scale()), &policy, &mut log)
        .expect("a rod with a square cross section encloses space");
    let held = across * across;
    assert!(
        measured.volume() > 0.0,
        "a rod holding {held:e} measured {}",
        measured.volume()
    );
}

/// `value` moved `steps` representable numbers along.
fn ulps_away(value: f64, steps: i64) -> f64 {
    f64::from_bits((value.to_bits() as i64 + steps) as u64)
}

/// The same numbers every run, so the counts above are numbers.
struct Seed(u64);

impl Seed {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 11
    }

    fn unit(&mut self) -> f64 {
        self.next() as f64 / (1u64 << 53) as f64
    }

    fn between_minus_one_and_one(&mut self) -> f64 {
        self.unit() * 2.0 - 1.0
    }
}
