//! A wire that stands still at a corner which is not where its ball pinches.
//!
//! A ball's face is bounded by one wire: down the seam, stand still at the
//! south pole, back up the seam, stand still at the north pole. The standing
//! still is a `DraftStep::Pole`, and the mesher trusts it — it squashes a whole
//! side of the face's `(u, v)` rectangle into that one corner. So a pole step
//! at a corner somewhere else draws a mesh folded over itself
//! (`docs/decisions.md` §188.2), and `Model::validate` now refuses it as
//! `Invalid::APoleStandsWhereTheSurfaceDoesNotPinch` before anything is meshed.
//!
//! **Not an exact check.** A pole is kept when the box the sweep already gives
//! a corner of a curved face — what writing a point down in `f64` costs, or
//! the slop recorded on the corner, whichever is wider — still holds a pinch.
//! The pairs of tests below sit either side of that room.
//!
//! # The seed it was written for
//!
//! §188.2 names fuse seed `3482a32e…`, whose first half the mesher used to
//! refuse after the fact. The sweep refuses it now, though for its *other*
//! corner, and as a corner off its ball rather than a pole off its pinch:
//! [`the_seed_that_prompted_this_is_refused_by_the_sweep`] says which, and why
//! the corner at the ball's centre is still inside the room.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use rcad_geom::{
    EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Result, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// The room the sweep gives a corner of a curved face when nothing is recorded
/// on it: `MEASURING_SLACK · NEAREST_F64_POINT_ROUNDOFF` times the largest
/// coordinate in the model, which for these balls is the radius.
const ROOM: f64 = 16.0 * f64::EPSILON * RADIUS;

/// A ball of radius five about the origin, with its north pole where it
/// belongs and its second pole step at `pole` — which the seam, the half of
/// the circle `y = 0` with `x >= 0`, runs down to from the north.
fn a_ball_whose_second_pole_is_at(pole: Point) -> Result<SolidKey> {
    let mut model = Model::new(Scale::millimetres(100.0 * RADIUS).expect("a working size"));
    let mut builder = SolidBuilder::new();
    let north = builder.add_vertex(Point::new(0.0, 0.0, RADIUS))?;
    let other = builder.add_vertex(pole)?;
    let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let facing_x = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis has length");
    let seam = builder.add_edge(
        north,
        other,
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(meridian),
            picked_by: smallvec::smallvec![Picker::from(facing_x)],
        },
    )?;
    let down = builder.add_coedge(seam, Orientation::Forward)?;
    let up = builder.add_coedge(seam, Orientation::Reversed)?;
    let wire = builder.add_wire(&[
        DraftStep::Along(down),
        DraftStep::Pole(other),
        DraftStep::Along(up),
        DraftStep::Pole(north),
    ])?;
    let skin = Sphere::new(Point::origin(), RADIUS).expect("a positive radius");
    let face = builder.add_face(Surface::Sphere(skin), &[wire])?;
    builder.add_shell(&[(face, Orientation::Forward)])?;
    builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn the_refusal_for(pole: Point) -> Invalid {
    match a_ball_whose_second_pole_is_at(pole) {
        Err(Error::Invalid(why)) => why,
        other => panic!("a ball whose pole is at {pole:?} came back {other:?}"),
    }
}

/// The control: the ball `Model::add_ball` builds, through the builder's door.
#[test]
fn a_ball_with_both_poles_where_it_pinches_builds() {
    a_ball_whose_second_pole_is_at(Point::new(0.0, 0.0, -RADIUS))
        .expect("a ball with its poles at its poles");
}

/// A pole on the equator: exactly on the ball, a whole radius from either
/// pinch.
#[test]
fn a_pole_on_the_equator_is_refused() {
    let on_the_equator = Point::new(RADIUS, 0.0, 0.0);
    assert_eq!(
        the_refusal_for(on_the_equator),
        Invalid::APoleStandsWhereTheSurfaceDoesNotPinch {
            at: on_the_equator,
            pinch: Some(Point::new(0.0, 0.0, RADIUS)),
        }
    );
}

/// The kernel's own balls, read back from the corpus, put their poles up to
/// `3.6e-15` off the pinch (§188.2). That is inside the room, and it builds.
#[test]
fn a_pole_a_few_ulps_off_the_pinch_builds() {
    const _: () = assert!(3.6e-15 < ROOM, "the corpus's worst pole is inside the room");
    a_ball_whose_second_pole_is_at(Point::new(3.6e-15, 0.0, -RADIUS))
        .expect("a pole as far off as the corpus's own balls put one");
}

/// The other side of the same room: four times it is a corner somebody chose.
///
/// Still inside the room the "a corner is on its face" check gives — it is
/// off the ball by only `x²` — so it is this check, and not that one, that
/// refuses it.
#[test]
fn a_pole_further_off_the_pinch_than_the_room_is_refused() {
    let past_the_room = Point::new(4.0 * ROOM, 0.0, -RADIUS);
    assert_eq!(
        the_refusal_for(past_the_room),
        Invalid::APoleStandsWhereTheSurfaceDoesNotPinch {
            at: past_the_room,
            pinch: Some(Point::new(0.0, 0.0, -RADIUS)),
        }
    );
}

fn the_fuse_corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/two_solids_fused_by_a_stranger")
}

/// Both halves of one fuse seed, each decoded into a model if it decodes.
fn the_two_models_in(bytes: &[u8]) -> [Option<Model>; 2] {
    let Some((_, rest)) = bytes.split_first() else {
        return [None, None];
    };
    let (first, second) = rest.split_at(rest.len() / 2);
    [first, second].map(|half| postcard::from_bytes::<Model>(half).ok())
}

/// §188.2's seed: refused by the sweep now, before the mesher folds it — and
/// refused as a corner off its ball, before the pole check is asked.
///
/// Its first half is one ball of radius 5 whose wire takes a pole step at each
/// of its two corners. **Neither is at a pole.** One corner stands by the
/// ball's centre, but the other stands at `x = 4.7e226`, and the room a corner
/// gets is measured against the largest coordinate in the model — about
/// `1.7e212` here. So the corner by the centre is inside the room of a pole,
/// and of the ball, and is kept.
///
/// The corner at `4.7e226` used to get through "a corner is on its face",
/// because squaring its widened box went through `gap * gap`: the `y` and `z`
/// sides straddle zero with a reach of `1.7e212`, so interval multiplication
/// gave each square a lower end of `-inf`, and the sum came back undecided
/// with an enclosure of `inf`. A square is never below zero, and squared as a
/// square the sum starts at `f64::MAX` and is decided outside the ball
/// (`docs/decisions.md` §189.1). The pole check, which caught it before, is
/// now never reached for it.
#[test]
fn the_seed_that_prompted_this_is_refused_by_the_sweep() {
    let bytes = std::fs::read(the_fuse_corpus().join("3482a32e1c9186370acb804a509b12d5cbaf20a7"))
        .expect("the seed is checked in");
    let [Some(first), _] = the_two_models_in(&bytes) else {
        panic!("the seed's first half decodes into a model");
    };
    let refusal = first
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("a ball whose poles are not at its pinches");
    assert!(
        matches!(refusal, Error::Invalid(Invalid::VertexOffItsFaceSurface)),
        "the seed's first half came back {refusal:?}"
    );
    let at = first
        .vertices()
        .map(|corner| corner.point())
        .max_by(|one, other| {
            one.largest_coordinate_magnitude()
                .total_cmp(&other.largest_coordinate_magnitude())
        })
        .expect("the seed's ball has corners");
    assert_eq!(
        at.coordinates(),
        [4.737909217226285e226, 1.727233711018889e-77, 5.0]
    );
    let ball =
        Sphere::new(Point::new(1.113623391098575e-308, 0.0, 0.0), RADIUS).expect("the seed's ball");
    let room = 16.0 * f64::EPSILON * at.coordinates()[0];
    let widened = rcad_predicates::Point::Constructed(at.coordinates().map(|coordinate| {
        rcad_predicates::Interval::around(coordinate, room).expect("a finite box")
    }));
    let exactly = rcad_predicates::SideOfSurface::side_of(
        &ball,
        rcad_predicates::Point::from(at.coordinates()),
    );
    let in_its_room = rcad_predicates::SideOfSurface::side_of(&ball, widened);
    assert_eq!(exactly, rcad_predicates::Decision::Pos);
    assert_eq!(
        in_its_room,
        rcad_predicates::Decision::Pos,
        "the corner widened into its room"
    );
}

/// Which models in the fuse fuzzer's corpus this check refuses, and the one of
/// them nothing refused before.
///
/// **One half of 926 is refused for a pole off its pinch.** It was 132 when
/// the check landed, and 131 of those were refused before it existed —
/// fuzz models with corners out past `1e10`, which something later in the
/// sweep turned away. Since the quadric predicates square an interval as a
/// square and scale down an evaluation that overflowed (`docs/decisions.md`
/// §189.1), those 131 are refused one check earlier, as
/// `VertexOffItsFaceSurface`: their far corners are now decided off the ball
/// rather than undecided. Every one of the 926 verdicts was compared before
/// and after that change, and those 131 are the only ones that moved;
/// `validated` stays at 681.
///
/// **The one is `e8fe8935…` half 0, and it is a true refusal.** One ball of
/// radius 5 whose largest coordinate is 5, with a pole step at
/// `(4.66e-10, 0, 5)`. The pinch is at `(0, 0, 5)`, and the room is
/// `16 · ε · 5 = 1.8e-14` — so that pole is about 26 000 rooms off, a corner
/// somebody chose rather than one that rounded. It meshed before only because
/// `4.66e-10` of fold is too small for the mesher's guard to notice.
///
/// Pinned by name for the ones whose corners are at an ordinary size, so a
/// second real refusal joining it fails here with its name.
#[test]
fn the_fuse_corpus_holds_one_pole_off_its_pinch_that_nothing_refused_before() {
    let listing = std::fs::read_dir(the_fuse_corpus()).expect("the corpus is checked in");
    let mut decoded = 0;
    let mut refused = 0;
    let mut at_an_ordinary_size = Vec::new();
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        let bytes = std::fs::read(&path).expect("a readable seed");
        for (half, model) in the_two_models_in(&bytes).into_iter().enumerate() {
            let Some(model) = model else { continue };
            decoded += 1;
            if let Err(Error::Invalid(Invalid::APoleStandsWhereTheSurfaceDoesNotPinch { .. })) =
                model.validate(&Policy::certified(), &mut ProvenanceLog::new())
            {
                refused += 1;
                let largest = model
                    .vertices()
                    .map(|corner| corner.point().largest_coordinate_magnitude())
                    .fold(0.0, f64::max);
                if largest < 1e6 {
                    let seed = path.file_name().expect("a seed has a name");
                    at_an_ordinary_size.push(format!("{} half {half}", seed.to_string_lossy()));
                }
            }
        }
    }
    println!("{decoded} models decoded, {refused} refused for a pole off its pinch");
    assert!(
        decoded >= 680,
        "only {decoded} halves decoded, so the corpus moved"
    );
    assert_eq!(refused, 1, "halves refused for a pole off its pinch");
    assert_eq!(
        at_an_ordinary_size,
        ["e8fe8935c39e5c8e588016b74b233db6a93282b7 half 0"],
        "the models refused for a pole off its pinch whose corners are all \
         within a million units"
    );
}
