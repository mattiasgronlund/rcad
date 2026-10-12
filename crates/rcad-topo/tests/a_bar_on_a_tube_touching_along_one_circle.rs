//! A round bar of radius 1 standing on a tube whose wall runs from radius 1 to
//! 2, so the two touch along one circle and nothing else: the bar's bottom
//! rim, which is the rim of the tube's hole.
//!
//! The tube runs from `z = 0` to `z = 2` and the bar from `z = 2` to `z = 4`.
//! Near that circle the union is two solids meeting along an edge with four
//! faces round it, which `docs/decisions.md` §80.1 refuses as
//! `ResultIsNotManifold`, and §80.9 keeps refusing. The union of the bar as
//! `Model::add_round_bar` builds it is refused so, `count: 4`.
//!
//! The bar is also built cut in two by a plane through its axis and united
//! again ([`Bar`]). That union keeps a corner on each rim wherever
//! the cut crossed it, as `assemble.rs` does on purpose (`what_is_left`), and
//! nothing of the bar passes through those corners any more. Queue row Q1571
//! guessed that such a bar on the tube would be answered as two solids, its
//! rim in arcs and the tube's rim whole, so no edge had four faces.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// Where the bar stands on the tube.
const RIM: f64 = 2.0;

const THE_BAR: f64 = PI * 2.0;

const THE_TUBE: f64 = PI * (4.0 - 1.0) * 2.0;

const EVERY_BAR: [Bar; 3] = [Bar::Whole, Bar::CutOnXIsZero, Bar::CutOnYIsZero];

/// **The bar and the tube cannot be united**, whichever is the argument and
/// however the bar was built: `ResultIsNotManifold { count: 4 }`, the bar's
/// bottom cap and wall and the tube's top and inner wall along the one circle.
#[test]
fn a_bar_standing_on_a_tubes_hole_cannot_be_united_with_it() {
    for bar in EVERY_BAR {
        for bar_first in [true, false] {
            let asked = format!("{bar:?}, the bar first {bar_first}");
            let (_, said) = the_bar_and_the_tube(bar, bar_first, Rule::Union);
            let Err(Error::ResultIsNotManifold { count }) = said else {
                panic!("{asked}: {said:?}");
            };
            assert_eq!(count, 4, "{asked}");
        }
    }
}

/// **The bar and the tube share nothing, and each less the other is itself**,
/// whichever is the argument and however the bar was built: no solid, the bar,
/// `2π`, and the tube, `6π`.
#[test]
fn a_bar_standing_on_a_tubes_hole_shares_nothing_with_it() {
    for bar in EVERY_BAR {
        for (rule, bar_first, by_hand) in [
            (Rule::Intersection, true, None),
            (Rule::Intersection, false, None),
            (Rule::Difference, true, Some(THE_BAR)),
            (Rule::Difference, false, Some(THE_TUBE)),
        ] {
            let asked = format!("{bar:?}, {rule:?}, the bar first {bar_first}");
            let (model, said) = the_bar_and_the_tube(bar, bar_first, rule);
            let solids = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{asked}: {broken:?}"));
            match by_hand {
                None => assert!(solids.is_empty(), "{asked}: {solids:?}"),
                Some(volume) => {
                    let [solid] = solids.as_slice() else {
                        panic!("{asked}: {solids:?}");
                    };
                    holds(&model, *solid, volume, &asked);
                }
            }
        }
    }
}

/// **A bar cut in two through its axis and united again keeps a corner on its
/// bottom rim where the cut crossed it, and nothing else there**:
/// `(0, ±1, 2)` cut on `x = 0` and `(±1, 0, 2)` cut on `y = 0`. That is the
/// shape Q1571 asked for.
#[test]
fn a_bar_cut_in_two_and_united_again_keeps_a_corner_where_the_cut_crossed_its_rim() {
    for (bar, corners) in [
        (Bar::CutOnXIsZero, [[0.0, -1.0, RIM], [0.0, 1.0, RIM]]),
        (Bar::CutOnYIsZero, [[-1.0, 0.0, RIM], [1.0, 0.0, RIM]]),
    ] {
        let mut model = a_model();
        let solid = bar.build(&mut model);
        assert_eq!(corners_on_its_bottom_rim(&model, solid), corners, "{bar:?}");
    }
}

/// How the bar of radius 1 from `z = 2` to `z = 4` is built.
#[derive(Clone, Copy, Debug)]
enum Bar {
    /// `Model::add_round_bar`, its rims whole circles.
    Whole,
    /// The bar less a block over `x < 0`, united with the bar's other half.
    CutOnXIsZero,
    /// The bar less a block over `y < 0`, united with the bar's other half.
    CutOnYIsZero,
}

impl Bar {
    fn build(self, model: &mut Model) -> SolidKey {
        let a_bar = |model: &mut Model| {
            model
                .add_round_bar(1.0, 2.0, &at(RIM))
                .expect("a bar of radius one")
        };
        let below = match self {
            Self::Whole => return a_bar(model),
            Self::CutOnXIsZero => [[-10.0, 0.0], [-10.0, 10.0], [-10.0, 10.0]],
            Self::CutOnYIsZero => [[-10.0, 10.0], [-10.0, 0.0], [-10.0, 10.0]],
        };
        let [one, other] = [a_bar(model), a_bar(model)];
        let [block, its_copy] = [a_box(model, below), a_box(model, below)];
        let one = the_one_solid(model, one, block, Rule::Difference);
        let other = the_one_solid(model, other, its_copy, Rule::Intersection);
        the_one_solid(model, one, other, Rule::Union)
    }
}

/// The bar and the tube fused by `rule`, the bar the argument when `bar_first`.
fn the_bar_and_the_tube(
    bar: Bar,
    bar_first: bool,
    rule: Rule,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let mut model = a_model();
    let tube = a_tube(&mut model);
    let bar = bar.build(&mut model);
    let [argument, tool] = if bar_first { [bar, tube] } else { [tube, bar] };
    let said = model
        .fuse(
            &[argument],
            &[tool],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    (model, said)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// A bar of radius 2 from `z = 0` to `z = 2` less a bar of radius 1 through it.
fn a_tube(model: &mut Model) -> SolidKey {
    let outer = model
        .add_round_bar(2.0, RIM, &Isometry::identity())
        .expect("a bar of radius two");
    let through = model
        .add_round_bar(1.0, 4.0, &at(-1.0))
        .expect("a bar of radius one");
    the_one_solid(model, outer, through, Rule::Difference)
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{rule:?}: {refused:?}"));
    let [solid] = fused.solids() else {
        panic!("{rule:?} is one solid: {:?}", fused.solids());
    };
    *solid
}

/// Where the sides of `solid`'s faces start on `z = 2`, each once, in order.
fn corners_on_its_bottom_rim(model: &Model, solid: SolidKey) -> Vec<[f64; 3]> {
    let mut corners: Vec<[f64; 3]> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .map(|side| side.start().point().coordinates())
        .filter(|&[.., z]| z == RIM)
        .collect();
    corners.sort_by(f64_triples_in_order);
    corners.dedup();
    corners
}

fn f64_triples_in_order(one: &[f64; 3], other: &[f64; 3]) -> core::cmp::Ordering {
    one.iter()
        .zip(other)
        .map(|(a, b)| a.total_cmp(b))
        .find(|order| order.is_ne())
        .unwrap_or(core::cmp::Ordering::Equal)
}

fn holds(model: &Model, solid: SolidKey, volume: f64, asked: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{asked}: {} ± {} against {volume}",
        measured.volume(),
        measured.volume_error()
    );
}

fn at(z: f64) -> Isometry {
    Isometry::translation(Vector::new(0.0, 0.0, z)).expect("a finite shift")
}

fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let rectangle =
        ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            z[1] - z[0],
            &Isometry::translation(Vector::new(0.0, 0.0, z[0])).expect("a finite shift"),
        )
        .expect("a positive height")
}
