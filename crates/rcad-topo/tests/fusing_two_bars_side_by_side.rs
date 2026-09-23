//! Two overlapping parallel bars, united — and what still stops it.
//!
//! Ordinary mechanical CAD: two round bars that run the same way and share some
//! material. §48 pinned one reason the fuse could not do it — `arrange.rs`'s
//! straight gate swept the whole cylinder-cylinder row into `false`, so a
//! crossing that `rcad-geom` builds exactly as two endless lines fell past the
//! straight gate, past the conic gate, and out as
//! `ACrossingThisCannotBeOrderedAlong` with nothing having run out of
//! arithmetic. That gap is closed, and
//! `two_pipes_side_by_side_run_up_their_common_axis` in `arrange.rs` measures
//! the crossing being ordered up `z` where it used to be refused.
//!
//! **The second reason is closed too, and this file moved with it.** A corner of
//! the answer is where a bar's flat cap meets *both* pipes, which is one plane
//! and two quadrics, and `meet_of_three_surfaces` used to want two of its three
//! flat. It now answers this triple exactly: two pipes running the same way
//! square `p` the same way, so their difference is a flat surface and the triple
//! is two planes and a quadric after all. The corners come back as `(3, ±4, 0)`,
//! whole numbers in boxes of no width, and
//! `a_cap_plane_against_two_side_by_side_pipes_meets_them_at_three_and_four`
//! below is where that is checked.
//!
//! **So the fuse now stops one gate further in, and this file's job is to say
//! which one.** It is no longer geometry at all: with the corners placed, the
//! cap disc is cut into pieces by the other pipe's wall, and `select.rs` gives
//! up on one of those pieces, undecided with an unbounded enclosure.
//!
//! **Which way it gives up depends on which bar is the tool, and until §217
//! nothing could see that.** `place_the_piece` raised one name for two different
//! searches (§216.4), so both orders read *"a point inside a piece of a face"*
//! and the pair looked symmetric. Split, they are not: one way round the winding
//! fan finds a point inside the piece and no solid can place it, the other way
//! round no point is found at all. A piece bounded partly by an arc being hard
//! to walk is still the missing capability §48 names in
//! `ACrossingThisCannotBeOrderedAlong`'s conic half and not the quartic one —
//! but it is only half of what stops this pair.
//!
//! Three refusals, in order, each one closed by a different unit: the straight
//! gate, then the three-surface meet, then `select.rs` on a curved piece.
//! The first two are behind us and both regressions would show up here by name.

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, Error as GeomError, Isometry, MeetOfThree, Plane, Point, Scale, Surface,
    Vector, meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

const RADIUS: f64 = 5.0;
const HEIGHT: f64 = 10.0;
/// How far apart the two axes are: less than `2·RADIUS`, so the bars overlap,
/// and more than `0`, so they are two bars and not one.
const APART: f64 = 6.0;

/// `radius = 5` and `apart = 6` is a 3-4-5 either side of the plane of the two
/// axes, so the two lines the pipes cut run through `(3, ±4, z)` and land on
/// coordinates an `f64` holds exactly. Nothing that refuses below is refusing
/// because a number would not fit.
fn two_bars_side_by_side() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let here = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius five, ten tall");
    let there = model
        .add_round_bar(
            RADIUS,
            HEIGHT,
            &Isometry::translation(Vector::new(APART, 0.0, 0.0)).expect("a shift along `x`"),
        )
        .expect("a second bar of the same size, six along `x`");
    (model, here, there)
}

/// **Where the union of two overlapping parallel bars stops today, by name.**
///
/// Not `.is_err()`: the whole worth of this test is *which* refusal it is. Two
/// of them are behind us and both are named below, because either coming back is
/// a regression rather than a curiosity — `ACrossingThisCannotBeOrderedAlong`
/// was the answer before §48's gate flip, and
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` was the answer until the
/// three-surface meet learned one plane and two quadrics.
///
/// What is left is `select.rs`, and since §217 the record says **which** of its
/// two searches gave out. It is not the same one both ways round, which is what
/// the single name was hiding; [`the_walls_measured_when_217_split_them`] is the
/// pin and says what was found.
///
/// All three rules and both orders of the pair, because a refusal that depended
/// on which solid was the tool would be a different bug.
#[test]
fn two_overlapping_parallel_bars_stop_in_select_whichever_way_round() {
    let policy = Policy::certified();
    let (mut model, here, there) = two_bars_side_by_side();
    let mut stopped_at: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (which, (arguments, tools)) in [
            ("here on there", ([here], [there])),
            ("there on here", ([there], [here])),
        ] {
            let mut log = ProvenanceLog::new();
            let refused = model.fuse(&arguments, &tools, rule, &policy, &mut log);
            assert!(
                !matches!(
                    refused,
                    Err(Error::Geometry(
                        GeomError::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                    )) | Err(Error::ACrossingThisCannotBeOrderedAlong)
                ),
                "{rule:?} on two side-by-side bars went back to a refusal that is closed: \
                 {refused:?}"
            );
            let Err(Error::Undecidable { records }) = &refused else {
                panic!("{rule:?} on two side-by-side bars answered {refused:?}");
            };
            for record in records.records() {
                *stopped_at
                    .entry(format!("{rule:?} {which}"))
                    .or_default()
                    .entry(record.predicate().to_owned())
                    .or_default() += 1;
            }
        }
    }
    assert_eq!(
        stopped_at,
        the_walls_measured_when_217_split_them(),
        "the side-by-side bars stopped somewhere else; re-derive this rather than widening it \
         to hold both names (§19.1)"
    );

    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a refused fuse leaves the model exactly as it was");
}

/// Which of `select.rs`'s two searches gave out, per rule and per order.
///
/// §216.4 found `place_the_piece` raising one name for both searches and §217
/// split them. This pair of bars reaches **both** halves, which is why the pin
/// is a map and not a string.
///
/// **What it splits on is the order, not the rule** — and that is the thing the
/// single name was hiding. Every rule behaves the same way round, and turning
/// the pair round changes which search gives out: `here` on `there` finds a
/// point inside the piece and cannot place it against both bars, `there` on
/// `here` never finds one at all. Both still refuse `Undecidable`, so the
/// assertion above about a closed refusal coming back is untouched, and the
/// module's "a refusal that depended on which solid was the tool would be a
/// different bug" still holds of the *refusal*. It is one step finer than that
/// sentence reaches: the pieces being placed are the argument's faces one way
/// round and the tool's the other, so the two searches are being asked about
/// different pieces. Whether that asymmetry is worth closing is not settled
/// here; §217 records it as found rather than diagnosed.
fn the_walls_measured_when_217_split_them() -> BTreeMap<String, BTreeMap<String, usize>> {
    let inside_the_piece = || BTreeMap::from([("a point inside a piece of a face".to_owned(), 1)]);
    let every_solid_could_place = || {
        BTreeMap::from([(
            "a point in a piece that every solid could place".to_owned(),
            1,
        )])
    };
    ["Union", "Intersection", "Difference"]
        .into_iter()
        .flat_map(|rule| {
            [
                (format!("{rule} here on there"), every_solid_could_place()),
                (format!("{rule} there on here"), inside_the_piece()),
            ]
        })
        .collect()
}

/// **The corner the fuse could not place, and now can, asked for on its own.**
///
/// A bar's cap plane against the two pipes: one flat surface and two quadrics.
/// `radius = 5` and `apart = 6` gives `x² + y² = 25` against
/// `(x − 6)² + y² = 25`, and subtracting one from the other leaves `12x = 36` —
/// so the corners are `(3, ±4, 0)`, a 3-4-5 with every number whole. Checked
/// against those numbers and against boxes of no width, so what is compared is
/// the whole answer and not a rounding of it.
///
/// This is the call `Arrangement::place_the_meets` makes and hands straight
/// back — the only place in this crate that lets that error out, since
/// `bake.rs`'s corner search swallows it and tries the next triple.
#[test]
fn a_cap_plane_against_two_side_by_side_pipes_meets_them_at_three_and_four() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("a bar's bottom cap"),
    );
    let here = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("the first pipe"));
    let there = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(APART, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                .expect("an axis six along `x`"),
            RADIUS,
        )
        .expect("the second pipe"),
    );

    let answer = meet_of_three_surfaces([&cap, &here, &there], &policy, &mut log)
        .expect("two pipes running the same way and a cap");
    let MeetOfThree::At(corners) = &answer else {
        panic!("expected two corners, got {answer:?}");
    };
    let mut placed: Vec<[f64; 3]> = corners
        .iter()
        .map(|corner| {
            corner.enclosure().map(|side| {
                assert_eq!(side.lo(), side.hi(), "{corner:?} is a box with width");
                side.lo()
            })
        })
        .collect();
    placed.sort_by(|one, other| one[1].total_cmp(&other[1]));
    assert_eq!(placed, vec![[3.0, -4.0, 0.0], [3.0, 4.0, 0.0]]);
    assert!(
        log.is_empty(),
        "a certified answer wrote {log:?} into the log"
    );
}
