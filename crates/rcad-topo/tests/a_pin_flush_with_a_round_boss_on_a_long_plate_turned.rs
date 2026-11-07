//! A thick round bar with a plate on one side, a thin bar touching the thick
//! bar's wall from inside, and a box on the plate's far end, all turned about
//! `z` (Q3247's flush pin).
//!
//! At `Scale::millimetres(100.0)` the thick bar has radius 2 and stands on the
//! `z` axis from `z = −2` to 2. The plate is `x` 0 to `L`, `y` −1.5 to 1.5,
//! `z` −2 to 2. The thin bar has radius 1 and stands from `z = −3` to 3 with
//! its axis at `(−1, 0)`, so it touches the thick bar's wall along one line.
//! The box is `x` `L` to `L + 2`, `y` −0.5 to 0.5, `z` −1 to 1, standing on
//! the plate's end. All four are turned about `z` by one angle and united in
//! that order: the plate, then the thin bar, then the box.
//!
//! Turned, the thin bar's pipe crosses the thick bar's in two lines `2e−8`
//! apart, from the floor to the top, and the strip of the thin bar's wall
//! between them is a face of what the box is united with. Each face of that
//! solid is cut by every surface of the box (`docs/decisions.md` §48), and
//! the box's planes `z = −1` and `z = 1` cut the strip in two arcs, each
//! running from one line to the other, though the box is `L` or more away.
//! So the strip from `z = −1` to 1 is a piece with all four corners on those
//! planes, and the strip from `z = 1` to 2 and the one from `z = −2` to `−1`
//! are pieces with two corners on them.
//!
//! No point can be found inside any of the three to place it by, and no ray
//! is fired from a corner on a plane of a face whose sides are all straight
//! (`by_a_corner` in `fuse/select.rs`). Each arc starts clear of every face of
//! the box, so the middle strip takes the placement of the piece across it
//! (`by_the_piece_across_a_side_clear_of_every_face`), but that rung only asked
//! the point search, and the union with the box was refused `Undecidable` at
//! `a point inside a piece of a face` (Q3262, Q3267). It now places the piece
//! across by a corner too, and the corners at `z = 2` or `−2`, off every
//! surface of the box, place it outside.
//!
//! **One solid or two.** Where the box's face at `x = L` lands against the
//! plate's end is a rounding of the turn. The two store one normal to the bit,
//! and the box's offset is the plate's, or an ulp or two either side of it. On
//! it, the two faces lie on each other facing opposite ways and neither is
//! kept. An ulp into the plate, its end cuts the box's sides and the sliver
//! inside goes. An ulp clear of it, the box meets nothing, and the union is the
//! two solids: the plate 50 long at the turns `0.4` and `2`, where the box's
//! face is stored at `50.00000000000001` and `50.000000000000014`, and the
//! plate 5 long at `4.7`, at `5.000000000000001`.

use core::f64::consts::PI;
use std::cmp::Ordering;

use rcad_geom::{Axis, Deflection, Isometry, Plane, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The turns at which the union with the box was refused on either plate:
/// Q3262's `0.1`, `1`, `2` and `4.7`, and Q3276's `0.2` and `0.4`.
const TURNS: [f64; 6] = [0.1, 0.2, 0.4, 1.0, 2.0, 4.7];

#[test]
fn a_box_on_a_plate_five_long_unites_with_a_thin_bar_flush_with_a_thick_one() {
    the_box_unites_at_every_turn(5.0);
}

#[test]
fn a_box_on_a_plate_fifty_long_unites_with_a_thin_bar_flush_with_a_thick_one() {
    the_box_unites_at_every_turn(50.0);
}

/// Panics unless, at each of [`TURNS`], the box unites with the thick bar,
/// the plate `length` long and the thin bar into one solid, or into two where
/// a rounding leaves it clear of the plate's end, which validate and measure
/// the four's volume.
fn the_box_unites_at_every_turn(length: f64) {
    let deflection = Deflection::of(0.02).expect("a mesh");
    for turn in TURNS {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let thick = a_bar(&mut model, 2.0, 4.0, [0.0, 0.0, -2.0], turn);
        let plate = a_block(
            &mut model,
            [0.0, -1.5],
            [length, 1.5],
            4.0,
            [0.0, 0.0, -2.0],
            turn,
        );
        let thin = a_bar(&mut model, 1.0, 6.0, [-1.0, 0.0, -3.0], turn);
        let the_box = a_block(
            &mut model,
            [-1.0, -0.5],
            [1.0, 0.5],
            2.0,
            [length + 1.0, 0.0, -1.0],
            turn,
        );
        let solids_meant = match where_the_box_stands(&model, plate, the_box) {
            Ordering::Less | Ordering::Equal => 1,
            Ordering::Greater => 2,
        };

        let with_the_plate = fused(&mut model, &[thick], &[plate], || {
            format!("turned {turn}, the plate {length} long")
        });
        let with_the_thin_bar = fused(&mut model, &with_the_plate, &[thin], || {
            format!("turned {turn}, the thin bar on the plate {length} long")
        });
        let last = || format!("turned {turn}, the box on the plate {length} long");
        let answer = fused(&mut model, &with_the_thin_bar, &[the_box], last);
        assert_eq!(answer.len(), solids_meant, "{}: how many solids", last());
        valid_and_of_volume(&model, &answer, the_volume(length), deflection, last);
    }
}

/// Where the box's face at `x = L` stands against the plate's end, read off
/// the planes the two store: [`Ordering::Equal`] on it, [`Ordering::Less`]
/// into the plate and [`Ordering::Greater`] clear of it.
fn where_the_box_stands(model: &Model, plate: SolidKey, the_box: SolidKey) -> Ordering {
    let end = planes_of(model, plate)
        .max_by(|one, other| one.offset().total_cmp(&other.offset()))
        .expect("a plate has faces");
    let face = planes_of(model, the_box)
        .map(|plane| plane.reversed())
        .find(|plane| plane.normal().components() == end.normal().components())
        .expect("the box's face at x = L stores the plate's end's normal reversed, to the bit");
    face.offset().total_cmp(&end.offset())
}

fn planes_of(model: &Model, solid: SolidKey) -> impl Iterator<Item = Plane> + '_ {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| *face.surface().as_plane().expect("a block's faces are flat"))
}

/// Panics, saying `which`, unless the model validates and `solids` together
/// measure `meant` at `deflection` to within the measures' own errors.
fn valid_and_of_volume(
    model: &Model,
    solids: &[SolidKey],
    meant: f64,
    deflection: Deflection,
    which: impl Fn() -> String,
) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{}: {why}", which()));
    let (volume, error) = solids.iter().fold((0.0, 0.0), |(volume, error), &solid| {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|refused| panic!("{}: {refused:?}", which()));
        (volume + measured.volume(), error + measured.volume_error())
    });
    println!(
        "{}: {} solids, {volume} ± {error} against {meant}",
        which(),
        solids.len()
    );
    assert!(
        (volume - meant).abs() <= error,
        "{}: {volume} ± {error} against {meant}",
        which()
    );
}

/// The thick bar and a plate `length` long, `4` tall: the disk of radius 2
/// and the rectangle `length × 3`, less the part of the disk with `x ≥ 0` and
/// `|y| ≤ 1.5`, whose area is `1.5 √1.75 + 4 asin 0.75`. The thin bar adds its
/// two ends of length 1 past `z = ±2`, and the box `2 × 1 × 2`.
fn the_volume(length: f64) -> f64 {
    let disk = 4.0 * PI;
    let rectangle = 3.0 * length;
    let both = 1.5 * 1.75_f64.sqrt() + 4.0 * 0.75_f64.asin();
    4.0 * (disk + rectangle - both) + 2.0 * PI + 4.0
}

/// `one` united with `other`, panicking with `which` when it is refused.
fn fused(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    which: impl Fn() -> String,
) -> Vec<SolidKey> {
    model
        .fuse(
            one,
            other,
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{}: {refused:?}", which()))
        .solids()
        .to_vec()
}

fn a_bar(model: &mut Model, radius: f64, height: f64, foot: [f64; 3], turn: f64) -> SolidKey {
    model
        .add_round_bar(radius, height, &turned(turn, foot))
        .expect("a bar")
}

/// The rectangle from `low` to `high` raised `height` from `foot`, turned.
fn a_block(
    model: &mut Model,
    low: [f64; 2],
    high: [f64; 2],
    height: f64,
    foot: [f64; 3],
    turn: f64,
) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![low, [high[0], low[1]], high, [low[0], high[1]]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(&rectangle, height, &turned(turn, foot))
        .expect("a block")
}

/// A move to `at`, then the turn about `z`.
fn turned(turn: f64, at: [f64; 3]) -> Isometry {
    Isometry::rotation(&Axis::z(), turn)
        .expect("a finite angle")
        .compose(&Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move"))
}
