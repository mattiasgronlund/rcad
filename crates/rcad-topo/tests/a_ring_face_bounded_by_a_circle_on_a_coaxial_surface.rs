//! A ring cut by a pipe, a cone or a ball on its axis, each of the two pieces
//! built face by face through [`SolidBuilder`] and walked every way. Row Q1173,
//! `docs/decisions.md` §243.8.
//!
//! The ring is Q1153's, of radius ten and tube five about `z`. Each partner
//! crosses the tube in two circles round the axis, and each piece is the ring
//! on one side of the partner, closed off by the partner between the circles:
//!
//! | partner | circles at `(r, z)` | the piece holding `(15, 0, 0)` | the other piece |
//! |---|---|---|---|
//! | the pipe of radius 13 | `(13, −4)`, `(13, 4)` | the band outside the pipe | the ring with its rim turned down to the pipe |
//! | the pipe of radius 7 | `(7, −4)`, `(7, 4)` | the ring with its bore turned out to the pipe | the band inside the pipe |
//! | the cone `r = 9 − z` | `(13, −4)`, `(6, 3)` | the ring beyond the cone | the ring within it |
//! | the ball of radius `√185` about the origin | `(13, −4)`, `(13, 4)` | the band outside the ball | the ring inside it |
//!
//! **Each circle's run names its own plane**, `z = −4`, `z = 4` or `z = 3`,
//! stored facing `+z`, which is how `rcad-step` reads a circle between two
//! different curved faces, and since row Q1334 how `intersect` names the
//! crossing of a ring with a pipe, a cone or a ball on its axis. A circle whose
//! run names only the faces either side is refused for the pair.
//!
//! **Before row Q1173, `Model::validate` refused five of the eight pieces
//! however they were walked**: the turned rim, the turned bore, both pieces of
//! the cone and the ring inside the ball. A run naming a plane between two
//! curved faces is read as the plane and the face that sorts first, which is
//! never the ring, so the ring's face was not held to leaving its two circles
//! going opposite ways round the tube (row Q1153), and the side of each plane
//! it keeps was read off its corners. Its one corner off that plane is the
//! other circle's, and on a piece that runs on round the tube past that plane
//! the corner is on the side the face does not keep: at `(13, 0, −4)` the
//! turned rim's other corner `(13, 0, 4)` read above `z = −4` where the face
//! leaves its circle going below it. The three bands' corners are on the side
//! they keep, and no walk of any piece passed that is not the piece.
//!
//! **Before row Q1335, asking the ring's face of any piece whether it holds a
//! point panicked**, and so did `classify_point` anywhere. The face is trimmed
//! by the plane each circle's run names, so it is read as a ring face between
//! two planes, and that reading took the face across each circle, the partner,
//! for the plane. Read off the run instead, the pipes and the ball leave the
//! band between the planes or the ring less it (`docs/decisions.md` §243.8),
//! and the cone leaves one outer circle on `z = −4` and one inner on `z = 3`:
//! one circle in front of the other's plane and the other behind it, which was
//! refused as no face before (row Q1155). Its face is the part outside the
//! middle of the tube on its side of `z = −4` and the part inside it on its
//! side of `z = 3`.

use core::cmp::Ordering;
use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere,
    Surface, SurfaceMeet, Torus, Vector, intersect,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftStep, Error, Invalid, Model, OnAFace, Result, SolidBuilder, SolidKey,
};

const EVERY_WALK: [[Orientation; 2]; 4] = [
    [Orientation::Forward, Orientation::Forward],
    [Orientation::Forward, Orientation::Reversed],
    [Orientation::Reversed, Orientation::Forward],
    [Orientation::Reversed, Orientation::Reversed],
];

/// `intersect` names a ring's crossing with each of the three partners as the
/// two circles, lower first, each by its own plane facing `+z`, either way
/// round. Row Q1334 (§378); before it every pair was refused
/// `ACrossingBetweenATorusAndAnotherCurvedSurface`.
///
/// For the pipes and the cone those are the runs [`a_piece_named_by_planes`]
/// builds every piece with, bit for bit. The ball's radius is a rounding of
/// `√185`, so its circles sit at `±3.99999999999999939`, worked out in exact
/// rationals, and their planes are roundings of that: held here to the
/// allowance `intersect` refuses past, sixteen roundings of the largest number.
#[test]
fn intersect_names_each_circle_of_a_ring_with_a_pipe_cone_or_ball_on_its_axis_by_its_plane() {
    let the_balls_circles = 3.999_999_999_999_999_4;
    for (partner, heights, allowed) in [
        (a_pipe(13.0), [-4.0, 4.0], 0.0),
        (a_pipe(7.0), [-4.0, 4.0], 0.0),
        (the_cone(), [-4.0, 3.0], 0.0),
        (
            the_ball(),
            [-the_balls_circles, the_balls_circles],
            16.0 * f64::EPSILON * 15.0,
        ),
    ] {
        for [one, other] in [[the_ring(), partner], [partner, the_ring()]] {
            let met = intersect(
                &one,
                &other,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{partner:?}: {why:?}"));
            let SurfaceMeet::Curves(runs) = &met else {
                panic!("{partner:?}: {met:?}");
            };
            assert_eq!(runs.len(), 2, "{partner:?}: {met:?}");
            for (run, up) in runs.iter().zip(heights) {
                let EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(flat),
                } = run.run()
                else {
                    panic!("{partner:?}: {run:?}");
                };
                assert_eq!(flat.normal(), Vector::new(0.0, 0.0, 1.0), "{partner:?}");
                assert!(
                    (flat.offset() - up).abs() <= allowed,
                    "{partner:?}: {} where the circle is at {up}",
                    flat.offset()
                );
            }
        }
    }
}

/// Every piece, walked any way, with each circle's run naming only the faces
/// either side, is refused for the pair those faces are.
///
/// **On purpose, and not for want of the arithmetic** since row Q1396: `walk`
/// draws an arc of such a circle, and refuses only a whole one named this way.
/// Answered, the pieces walked their own way held what Pappus gives, but the
/// band outside the pipe of 13 with both circles walked forwards — a
/// description that bounds no piece, refused
/// `AClosedEdgeIsWalkedAgainstItsSurface` in
/// [`every_other_description_of_each_piece_is_refused`] — built and validated,
/// because `Model::validate` holds the ring's face to its band by the plane the
/// run names (§243.8), and these runs name none.
#[test]
fn a_circle_named_only_by_the_ring_and_its_partner_is_refused_for_the_pair() {
    for piece in every_piece() {
        for walk in EVERY_WALK {
            for wall_used in [Orientation::Forward, Orientation::Reversed] {
                let built = a_piece_built(
                    &piece,
                    [
                        EdgeRun::RightRoundWhereItsFacesCross,
                        EdgeRun::RightRoundWhereItsFacesCross,
                    ],
                    walk,
                    wall_used,
                );
                assert!(
                    matches!(
                        built,
                        Err(Error::Geometry(
                            rcad_geom::Error::ACrossingBetweenATorusAndAnotherCurvedSurface
                        ))
                    ),
                    "{} walked {walk:?}, wall used {wall_used:?}: {:?}",
                    piece.name,
                    built.map(|_| ())
                );
            }
        }
    }
}

/// Each piece, walked its own way with the ring used as stored, builds, holds
/// what Pappus gives it within its stated error, and still validates moved a
/// hundred along `x`.
///
/// Five of the eight were refused `AClosedEdgeIsWalkedAgainstItsSurface` before
/// row Q1173 (the file's own doc says why), and the bands outside the pipe of
/// 13 and the ball and inside the pipe of 7 were not.
#[test]
fn each_piece_walked_its_own_way_validates_and_holds_what_pappus_gives() {
    for piece in every_piece() {
        let (mut model, solid) = a_piece_named_by_planes(&piece, piece.walk, Orientation::Forward)
            .unwrap_or_else(|why| panic!("{}: {why:?}", piece.name));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{}: {why:?}", piece.name));
        let measured = model
            .solid(solid)
            .expect("its own key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{}: {why:?}", piece.name));
        assert!(
            (measured.volume() - piece.volume).abs() <= measured.volume_error(),
            "{}: measured {} where Pappus gives {}, allowed {}",
            piece.name,
            measured.volume(),
            piece.volume,
            measured.volume_error()
        );
        model
            .place_solid(
                solid,
                &Isometry::translation(Vector::new(100.0, 0.0, 0.0)).expect("a finite shift"),
            )
            .unwrap_or_else(|why| panic!("{} moved: {why:?}", piece.name));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{} moved: {why:?}", piece.name));
    }
}

/// Every other description of each piece is refused: walked any other way,
/// with the ring used either way, `AClosedEdgeIsWalkedAgainstItsSurface`, and
/// walked its own way with the ring used turned round, the piece turned inside
/// out, `VolumeIsNotPositive`.
///
/// The other ways are the two that walk both circles the same way round the
/// tube and bound no piece, and the one turned over at both, which is the other
/// piece's wall closed off by this piece's partner. Before row Q1173 the three
/// bands' own walk with the ring turned round was refused
/// `AClosedEdgeIsWalkedAgainstItsSurface` off a corner rather than by the mesh.
#[test]
fn every_other_description_of_each_piece_is_refused() {
    for piece in every_piece() {
        for walk in EVERY_WALK {
            for wall_used in [Orientation::Forward, Orientation::Reversed] {
                if walk == piece.walk && wall_used == Orientation::Forward {
                    continue;
                }
                let built = a_piece_named_by_planes(&piece, walk, wall_used).map(|_| ());
                let as_expected = if walk == piece.walk {
                    matches!(
                        built,
                        Err(Error::Invalid(Invalid::VolumeIsNotPositive { .. }))
                    )
                } else {
                    matches!(
                        built,
                        Err(Error::Invalid(
                            Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                        ))
                    )
                };
                assert!(
                    as_expected,
                    "{} walked {walk:?}, wall used {wall_used:?}: {built:?}",
                    piece.name
                );
            }
        }
    }
}

/// The ring's face of each piece cut by a pipe or the ball holds the part of
/// the ring on the piece's side of the partner, has the two circles as its
/// boundary and holds nothing else, wherever round the axis it is asked, as
/// built and moved a hundred along `x`.
///
/// Each circle's run names its own plane, so the face is trimmed by two planes
/// square to the axis, and here its two circles are both outside the middle of
/// the tube or both inside it: the band between the planes, or the ring less
/// that band (`docs/decisions.md` §243.8). Before row Q1335 every point
/// panicked, because the face across each circle, the partner, was taken for
/// the plane the circle is cut by.
#[test]
fn the_ring_face_of_each_piece_cut_by_a_pipe_or_the_ball_holds_its_side_of_the_partner() {
    for piece in every_piece() {
        if !piece.has_one_circle_each_side_of_the_middle_of_the_tube() {
            the_ring_face_holds_its_side_of_the_partner(&piece);
        }
    }
}

/// The same of the two pieces the cone cuts, whose ring face has one circle
/// outside the middle of the tube on `z = −4` and one inside it on `z = 3`.
///
/// Each circle is in front of the other's plane on one side and behind it on
/// the other, which is neither a band between two planes nor the ring less one.
/// The face is the outer part of the ring on its side of `z = −4` and the inner
/// part on its side of `z = 3`, which meet at the top or the bottom of the tube.
/// Before row Q1335 every point panicked, as above, and with the planes read
/// right the face was refused `AFaceMissesPartOfWhereItsSurfacesCross` (row
/// Q1155).
#[test]
fn the_ring_face_of_each_piece_cut_by_the_cone_holds_its_side_of_the_cone() {
    for piece in every_piece() {
        if piece.has_one_circle_each_side_of_the_middle_of_the_tube() {
            the_ring_face_holds_its_side_of_the_partner(&piece);
        }
    }
}

/// `classify_point` finds each piece cut by a pipe or the ball inside the tube
/// on its side of the partner, on its skin where the ring's face or the
/// partner's holds the point, and outside everywhere else. Before row Q1335
/// every point panicked, at the ring's face.
#[test]
fn classify_point_finds_each_piece_cut_by_a_pipe_or_the_ball_on_its_side_of_the_partner() {
    for piece in every_piece() {
        if !piece.has_one_circle_each_side_of_the_middle_of_the_tube() {
            classify_point_finds_the_piece_on_its_side_of_the_partner(&piece);
        }
    }
}

/// The same of the two pieces the cone cuts.
#[test]
fn classify_point_finds_each_piece_cut_by_the_cone_on_its_side_of_the_cone() {
    for piece in every_piece() {
        if piece.has_one_circle_each_side_of_the_middle_of_the_tube() {
            classify_point_finds_the_piece_on_its_side_of_the_partner(&piece);
        }
    }
}

/// Points on the ring, at `(r, z)`, each a whole number from the tube's middle
/// circle: the outside, the inside, the top and the bottom of the tube, the
/// four points of the planes `z = ±3` and `z = ±4`, which hold every piece's
/// circles.
const ON_THE_RING: [(f64, f64); 12] = [
    (15.0, 0.0),
    (5.0, 0.0),
    (10.0, 5.0),
    (10.0, -5.0),
    (14.0, 3.0),
    (14.0, -3.0),
    (6.0, 3.0),
    (6.0, -3.0),
    (13.0, 4.0),
    (13.0, -4.0),
    (7.0, 4.0),
    (7.0, -4.0),
];

/// Points inside the tube, at `(r, z)`: three of them on a partner, `r = 13`
/// and `r = 7` on the pipes and `r = 9` at `z = 0` on the cone.
const IN_THE_TUBE: [(f64, f64); 9] = [
    (14.0, 0.0),
    (12.0, 0.0),
    (8.0, 0.0),
    (6.0, 0.0),
    (10.0, 4.0),
    (10.0, -4.0),
    (13.0, 0.0),
    (9.0, 0.0),
    (7.0, 0.0),
];

/// Asks `piece`'s ring face about every point of [`ON_THE_RING`] at three
/// places round the axis, and again with the piece moved a hundred along `x`.
fn the_ring_face_holds_its_side_of_the_partner(piece: &Piece) {
    let (mut model, solid) = a_piece_named_by_planes(piece, piece.walk, Orientation::Forward)
        .unwrap_or_else(|why| panic!("{}: {why:?}", piece.name));
    for shift in [0.0, 100.0] {
        model
            .place_solid(
                solid,
                &Isometry::translation(Vector::new(shift, 0.0, 0.0)).expect("a finite shift"),
            )
            .unwrap_or_else(|why| panic!("{} moved: {why:?}", piece.name));
        let view = model.solid(solid).expect("its own key");
        let wall = view
            .faces()
            .find(|face| matches!(face.surface(), Surface::Torus(_)))
            .expect("a face on the ring")
            .key();
        for (r, z) in ON_THE_RING {
            let expected = match piece.side_of_the_partner((r, z)) {
                Ordering::Equal => OnAFace::OnItsBoundary,
                side if side == piece.side_it_keeps() => OnAFace::Held,
                Ordering::Less | Ordering::Greater => OnAFace::NotHeld,
            };
            for at in round_the_axis((r, z), shift) {
                let held = view.face_holds_the_point(
                    wall,
                    PredicatePoint::from(at.coordinates()),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                assert!(
                    matches!(held, Ok(found) if found == expected),
                    "{} moved {shift}: the ring's face at {:?} said {held:?}, not {expected:?}",
                    piece.name,
                    at.coordinates()
                );
            }
        }
    }
}

/// Classifies every point of [`IN_THE_TUBE`] and [`ON_THE_RING`] at three
/// places round the axis, and again with the piece moved a hundred along `x`.
fn classify_point_finds_the_piece_on_its_side_of_the_partner(piece: &Piece) {
    let (mut model, solid) = a_piece_named_by_planes(piece, piece.walk, Orientation::Forward)
        .unwrap_or_else(|why| panic!("{}: {why:?}", piece.name));
    for shift in [0.0, 100.0] {
        model
            .place_solid(
                solid,
                &Isometry::translation(Vector::new(shift, 0.0, 0.0)).expect("a finite shift"),
            )
            .unwrap_or_else(|why| panic!("{} moved: {why:?}", piece.name));
        let view = model.solid(solid).expect("its own key");
        let in_the_tube = IN_THE_TUBE.map(|at| (at, Containment::Inside));
        let on_the_ring = ON_THE_RING.map(|at| (at, Containment::OnBoundary));
        for ((r, z), on_its_side) in in_the_tube.into_iter().chain(on_the_ring) {
            let expected = match piece.side_of_the_partner((r, z)) {
                Ordering::Equal => Containment::OnBoundary,
                side if side == piece.side_it_keeps() => on_its_side,
                Ordering::Less | Ordering::Greater => Containment::Outside,
            };
            for at in round_the_axis((r, z), shift) {
                let found =
                    view.classify_point(at, &Policy::certified(), &mut ProvenanceLog::new());
                assert_eq!(
                    found,
                    expected,
                    "{} moved {shift}: {:?}",
                    piece.name,
                    at.coordinates()
                );
            }
        }
    }
}

/// The point at `(r, z)` a quarter-turn and a half-turn apart round the axis,
/// moved `shift` along `x`.
fn round_the_axis((r, z): (f64, f64), shift: f64) -> [Point; 3] {
    [
        Point::new(shift + r, 0.0, z),
        Point::new(shift, r, z),
        Point::new(shift - r, 0.0, z),
    ]
}

/// One piece of the ring cut by a partner.
struct Piece {
    name: &'static str,
    partner: Surface,
    /// Each circle's corner, at `y = 0`, as `(r, z)`.
    circles: [(f64, f64); 2],
    /// The partner is used as stored where the piece is on the side of it its
    /// gradient points away from.
    partner_used: Orientation,
    /// How the ring's face walks the two circles, in the order of `circles`.
    walk: [Orientation; 2],
    /// What Pappus gives.
    volume: f64,
    /// How far past the partner the point at `(r, z)` is, up to a positive
    /// factor: above zero on the side its gradient points to. Worked out from
    /// the partner's own equation in whole numbers, so exactly, and asked only
    /// below the cone's apex.
    past_the_partner: fn((f64, f64)) -> f64,
}

impl Piece {
    /// Whether the piece's two circles are one outside the middle circle of the
    /// tube, `r = 10`, and one inside it, which only the cone's are.
    fn has_one_circle_each_side_of_the_middle_of_the_tube(&self) -> bool {
        let [(one, _), (other, _)] = self.circles;
        (one > 10.0) != (other > 10.0)
    }

    /// Which side of the partner the point at `(r, z)` is on, `Greater` where
    /// its gradient points.
    fn side_of_the_partner(&self, at: (f64, f64)) -> Ordering {
        (self.past_the_partner)(at)
            .partial_cmp(&0.0)
            .expect("a whole number")
    }

    /// The side of the partner the piece is on: the one its gradient points to
    /// where the piece uses it turned round.
    fn side_it_keeps(&self) -> Ordering {
        match self.partner_used {
            Orientation::Forward => Ordering::Less,
            Orientation::Reversed => Ordering::Greater,
            other => panic!("an orientation nothing here knows: {other:?}"),
        }
    }
}

/// The eight pieces, each worked out by slicing the ring across its axis or,
/// for the cone, by the segment of the tube's circle the line `r + z = 9`
/// cuts off.
///
/// Between `z = −4` and `z = 4`, the ring's slice at height `z` is the ring
/// between radii `10 ± s` with `s = √(25 − z²)`, and the band outside the pipe
/// of 13 is the slice beyond 13, `π((10 + s)² − 169)`. Summed over the slices,
/// each band comes to `π` times `20 I` less a polynomial, with `I` the area of
/// the tube's circle between `z = ±4`. The cone's line is `1/√2` from the tube's
/// middle, on the side away from the piece within the cone, whose area is `A`.
/// The other piece of each is the whole ring, `2π²·10·25`, less this one.
fn every_piece() -> [Piece; 8] {
    let whole = 2.0 * PI * PI * 10.0 * 25.0;
    let tube_between_the_planes = 12.0 + 25.0 * 0.8f64.asin();
    let outside_the_pipe_of_13 = PI * (20.0 * tube_between_the_planes - 352.0 - 128.0 / 3.0);
    let inside_the_pipe_of_7 = PI * (20.0 * tube_between_the_planes - 608.0 + 128.0 / 3.0);
    let outside_the_ball = PI * (20.0 * tube_between_the_planes - 480.0);
    let off_the_middle = 0.5f64.sqrt();
    let half_the_chord_squared: f64 = 25.0 - 0.5;
    let segment =
        25.0 * (off_the_middle / 5.0).acos() - off_the_middle * half_the_chord_squared.sqrt();
    let within_the_cone =
        2.0 * PI * (10.0 * segment - 2.0f64.sqrt() / 3.0 * half_the_chord_squared.powf(1.5));
    let (forward, reversed) = (Orientation::Forward, Orientation::Reversed);
    [
        Piece {
            name: "the band outside the pipe of 13",
            partner: a_pipe(13.0),
            circles: [(13.0, -4.0), (13.0, 4.0)],
            partner_used: reversed,
            walk: [forward, reversed],
            volume: outside_the_pipe_of_13,
            past_the_partner: past_the_pipe_of_13,
        },
        Piece {
            name: "the ring with its rim turned down to the pipe of 13",
            partner: a_pipe(13.0),
            circles: [(13.0, -4.0), (13.0, 4.0)],
            partner_used: forward,
            walk: [reversed, forward],
            volume: whole - outside_the_pipe_of_13,
            past_the_partner: past_the_pipe_of_13,
        },
        Piece {
            name: "the band inside the pipe of 7",
            partner: a_pipe(7.0),
            circles: [(7.0, -4.0), (7.0, 4.0)],
            partner_used: forward,
            walk: [reversed, forward],
            volume: inside_the_pipe_of_7,
            past_the_partner: past_the_pipe_of_7,
        },
        Piece {
            name: "the ring with its bore turned out to the pipe of 7",
            partner: a_pipe(7.0),
            circles: [(7.0, -4.0), (7.0, 4.0)],
            partner_used: reversed,
            walk: [forward, reversed],
            volume: whole - inside_the_pipe_of_7,
            past_the_partner: past_the_pipe_of_7,
        },
        Piece {
            name: "the ring beyond the cone",
            partner: the_cone(),
            circles: [(13.0, -4.0), (6.0, 3.0)],
            partner_used: reversed,
            walk: [reversed, forward],
            volume: whole - within_the_cone,
            past_the_partner: past_the_cone,
        },
        Piece {
            name: "the ring within the cone",
            partner: the_cone(),
            circles: [(13.0, -4.0), (6.0, 3.0)],
            partner_used: forward,
            walk: [forward, reversed],
            volume: within_the_cone,
            past_the_partner: past_the_cone,
        },
        Piece {
            name: "the band outside the ball",
            partner: the_ball(),
            circles: [(13.0, -4.0), (13.0, 4.0)],
            partner_used: reversed,
            walk: [forward, reversed],
            volume: outside_the_ball,
            past_the_partner: past_the_ball,
        },
        Piece {
            name: "the ring inside the ball",
            partner: the_ball(),
            circles: [(13.0, -4.0), (13.0, 4.0)],
            partner_used: forward,
            walk: [reversed, forward],
            volume: whole - outside_the_ball,
            past_the_partner: past_the_ball,
        },
    ]
}

fn the_ring() -> Surface {
    Surface::Torus(Torus::new(Axis::z(), 10.0, 5.0).expect("a tube narrower than its ring"))
}

fn a_pipe(radius: f64) -> Surface {
    Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"))
}

/// The cone `r = 9 − z`: apex `(0, 0, 9)`, widening by one for every unit down.
fn the_cone() -> Surface {
    Surface::Cone(
        Cone::new(
            Axis::new(Point::new(0.0, 0.0, 9.0), Vector::new(0.0, 0.0, -1.0))
                .expect("a direction with a length"),
            1.0,
        )
        .expect("a positive slope"),
    )
}

/// The ball about the origin through `(13, 0, ±4)`, whose radius `√185` is
/// rounded; each corner is off it by that rounding, which the sweep allows.
fn the_ball() -> Surface {
    Surface::Sphere(Sphere::new(Point::origin(), 185f64.sqrt()).expect("a positive radius"))
}

fn past_the_pipe_of_13((r, _): (f64, f64)) -> f64 {
    r * r - 169.0
}

fn past_the_pipe_of_7((r, _): (f64, f64)) -> f64 {
    r * r - 49.0
}

/// Below the apex, the cone widens out the way its gradient points.
fn past_the_cone((r, z): (f64, f64)) -> f64 {
    r - (9.0 - z)
}

/// The ball as `√185` would be, not as it is rounded: no point asked is near
/// enough to it for the rounding to matter but the corners, and those are on
/// the planes that bound the ring's face there.
fn past_the_ball((r, z): (f64, f64)) -> f64 {
    r * r + z * z - 185.0
}

/// [`a_piece_built`] with each circle's run naming its own plane.
fn a_piece_named_by_planes(
    piece: &Piece,
    walk: [Orientation; 2],
    wall_used: Orientation,
) -> Result<(Model, SolidKey)> {
    let runs = piece.circles.map(|(_, up)| a_circles_run(up));
    a_piece_built(piece, runs, walk, wall_used)
}

/// The run of the circle round the axis at height `up`: its own plane, facing
/// `+z`.
fn a_circles_run(up: f64) -> EdgeRun {
    EdgeRun::RightRoundWhereItCrosses {
        and: Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, up), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        ),
    }
}

/// The ring's face bounded by `piece`'s two circles, each walked as `walk`
/// says, and the partner's face between them walking each the other way, in a
/// model of its own.
fn a_piece_built(
    piece: &Piece,
    runs: [EdgeRun; 2],
    walk: [Orientation; 2],
    wall_used: Orientation,
) -> Result<(Model, SolidKey)> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let mut builder = SolidBuilder::new();
    let mut wall_wires = Vec::with_capacity(2);
    let mut partner_wires = Vec::with_capacity(2);
    for (((r, up), run), turn) in piece.circles.into_iter().zip(runs).zip(walk) {
        let corner = builder.add_vertex(Point::new(r, 0.0, up))?;
        let circle = builder.add_edge(corner, corner, run)?;
        let on_the_ring = builder.add_coedge(circle, turn)?;
        wall_wires.push(builder.add_wire(&[DraftStep::Along(on_the_ring)])?);
        let on_the_partner = builder.add_coedge(circle, turn.reversed())?;
        partner_wires.push(builder.add_wire(&[DraftStep::Along(on_the_partner)])?);
    }
    let wall = builder.add_face(the_ring(), &wall_wires)?;
    let partner = builder.add_face(piece.partner, &partner_wires)?;
    builder.add_shell(&[(wall, wall_used), (partner, piece.partner_used)])?;
    let solid = builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())?;
    Ok((model, solid))
}
