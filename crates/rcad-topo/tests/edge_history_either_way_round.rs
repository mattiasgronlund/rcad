//! What `Fused` says became of an input edge or corner (§39.4), in two shapes.
//!
//! **A ball kept whole as a hollow** (queue row Q2114). A block less a ball
//! that touches nothing keeps the ball's one face whole, walked down a seam
//! between two poles as the ball's own face is. The ball's seam and both its
//! poles come through as that seam and those poles, as every edge and corner
//! of a box in the same place does. They used to read as deleted: a pole is
//! not an input corner to the arrangement, and a seam is never one of its
//! boundaries (`bake`'s header), so nothing carried their names across.
//! The answer's seam and poles are its own, not the input's (§38.1), so a
//! turned ball keeps only what its turn leaves on them.
//!
//! **A bar kept whole keeps its rims and their corners, but not its seam**
//! (queue row Q2126). The answer writes a bar's wall that nothing cuts as a
//! band between its two rims, with no seam (`Assembly::needs_no_seam`,
//! `docs/decisions.md` §36.12), so no edge of the answer runs where the
//! input's seam ran, and history, which names only the answer's edges, reads
//! it as deleted. A ball is not like this: the answer must cut a whole ball
//! open along a seam of its own, and the input's seam lands on it. A bar
//! standing clear of a wall loses its seam the same way
//! (`a_bar_on_a_wall_both_turned_by_a_float_angle_answer_every_rule`, Q1765).
//! Writing the seam back would give a wall nothing cuts a topology no other
//! band has, for a history nothing outside the tests reads (§13).
//!
//! **A block standing flush on a bigger block**: edge history does not depend on
//! which solid is handed to `fuse` first, even where the touch is a whole
//! *face* and not a line (queue row Q809).
//!
//! Q801's own tests pinned argument-order independence for a round bar
//! touching a wall along a line. Nobody had built a plain block standing on
//! a bigger block, where a cut on one block's top face runs along the other
//! block's own boundary edge — the case `Edges::of`'s doc comment names
//! directly ("a wall along a bar's seam, where the wall's side names nothing
//! and the skin's names the seam").
//!
//! The upper block is two wide, four deep, standing on a four-wide,
//! four-deep, four-high lower block: flush with the lower block's wall at
//! `x = 0` and matching its full depth in `y`, but stopping short of the
//! lower block's far wall at `x = 4`. United, whichever block is handed over
//! first:
//! - the lower block's edge at `x = 4` survives untouched — the upper block
//!   never reaches it;
//! - the upper block's edge at `x = 2` survives untouched too — the cut this
//!   makes in the lower block's top face runs exactly along this
//!   pre-existing edge of the upper block, so its history is the upper
//!   block's own edge and not a fresh line the arrangement invented, and the
//!   lower block names nothing there at all;
//! - the lower and upper blocks' walls at `x = 0` are the same flat plane
//!   from the lower block's foot to the upper block's head, so they merge
//!   into one face and the edge each block had along `x = 0` is deleted;
//! - the lower block's edges at `y = 0` and `y = 4` are cut short at `x = 2`,
//!   where the upper block's corner sits: only the stretch from `x = 2` to
//!   `x = 4` survives, and the stretch from `x = 0` to `x = 2` merges away
//!   with the upper block's own edges there, which are deleted outright.

use rcad_geom::{Axis, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{EdgeKey, Fused, Model, ProfileBuilder, Rule, SolidKey, VertexKey};

#[test]
fn a_hollow_ball_keeps_its_seam_and_poles() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = a_box(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(5.0, 5.0, 2.0)).expect("a whole-number shift"),
        )
        .expect("a ball of radius 1");
    let &[(seam, _)] = located_edges_of(&model, ball).as_slice() else {
        panic!("a ball has one edge, its seam");
    };
    let north = the_vertex_at(&model, ball, [5.0, 5.0, 3.0]);
    let south = the_vertex_at(&model, ball, [5.0, 5.0, 1.0]);

    let what = "a block less a ball inside it";
    let answer = model
        .fuse(
            &[block],
            &[ball],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} refused: {why}"));

    let mut wrong: Vec<String> = Vec::new();
    check_survives(
        &model,
        &answer,
        seam,
        [[5.0, 5.0, 1.0], [5.0, 5.0, 3.0]],
        "the ball's seam",
        what,
        &mut wrong,
    );
    check_corner_survives(
        &model,
        &answer,
        north,
        [5.0, 5.0, 3.0],
        "the ball's north pole",
        what,
        &mut wrong,
    );
    check_corner_survives(
        &model,
        &answer,
        south,
        [5.0, 5.0, 1.0],
        "the ball's south pole",
        what,
        &mut wrong,
    );
    assert!(
        wrong.is_empty(),
        "a ball kept whole as a hollow keeps its seam and its poles:\n{}",
        wrong.join("\n")
    );
}

/// The answer cuts a ball kept whole open along a seam of its own, the `+x`
/// half of the meridian square to `y`, with its poles straight up and straight
/// down, however the ball was turned (§38.1). Turned upside down, the ball's
/// north pole is the answer's bottom pole and its seam still runs down the
/// `+x` half. Turned a quarter about `z`, both poles stay where they were, but
/// the seam runs down the `+y` half, which is no edge of the answer.
#[test]
fn a_turned_hollow_ball_keeps_what_still_lands_on_the_answers_poles_and_seam() {
    let mut wrong: Vec<String> = Vec::new();
    for (axis, turn, quarters, seam_kept) in [("x", Axis::x(), 2, true), ("z", Axis::z(), 1, false)]
    {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = a_box(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
        let placement = Isometry::translation(Vector::new(5.0, 5.0, 2.0))
            .expect("a whole-number shift")
            .compose(&Isometry::quarter_turns(&turn, quarters).expect("a turn about an axis"));
        let ball = model.add_ball(1.0, &placement).expect("a ball of radius 1");
        let &[(seam, _)] = located_edges_of(&model, ball).as_slice() else {
            panic!("a ball has one edge, its seam");
        };
        let top = the_vertex_at(&model, ball, [5.0, 5.0, 3.0]);
        let bottom = the_vertex_at(&model, ball, [5.0, 5.0, 1.0]);

        let what = format!("a block less a ball turned {quarters} quarters about {axis}");
        let answer = model
            .fuse(
                &[block],
                &[ball],
                Rule::Difference,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} refused: {why}"));

        if seam_kept {
            check_survives(
                &model,
                &answer,
                seam,
                [[5.0, 5.0, 1.0], [5.0, 5.0, 3.0]],
                "the ball's seam",
                &what,
                &mut wrong,
            );
        } else {
            check_deleted(&model, &answer, seam, "the ball's seam", &what, &mut wrong);
        }
        check_corner_survives(
            &model,
            &answer,
            top,
            [5.0, 5.0, 3.0],
            "the ball's pole on top",
            &what,
            &mut wrong,
        );
        check_corner_survives(
            &model,
            &answer,
            bottom,
            [5.0, 5.0, 1.0],
            "the ball's pole underneath",
            &what,
            &mut wrong,
        );
    }
    assert!(
        wrong.is_empty(),
        "a turned ball kept whole keeps what lands on the answer's poles and seam:\n{}",
        wrong.join("\n")
    );
}

/// A ball turned a quarter about `x` has its poles at `(5, 4, 2)` and
/// `(5, 6, 2)`, on the answer's skin halfway between its poles, where the
/// answer has no corner, and its seam runs where the answer has no edge.
#[test]
fn a_hollow_ball_lying_on_its_side_keeps_neither_its_poles_nor_its_seam() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = a_box(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let placement = Isometry::translation(Vector::new(5.0, 5.0, 2.0))
        .expect("a whole-number shift")
        .compose(&Isometry::quarter_turns(&Axis::x(), 1).expect("a turn about x"));
    let ball = model.add_ball(1.0, &placement).expect("a ball of radius 1");
    let &[(seam, _)] = located_edges_of(&model, ball).as_slice() else {
        panic!("a ball has one edge, its seam");
    };
    let poles = [
        the_vertex_at(&model, ball, [5.0, 4.0, 2.0]),
        the_vertex_at(&model, ball, [5.0, 6.0, 2.0]),
    ];

    let what = "a block less a ball lying on its side";
    let answer = model
        .fuse(
            &[block],
            &[ball],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} refused: {why}"));

    let mut wrong: Vec<String> = Vec::new();
    check_deleted(&model, &answer, seam, "the ball's seam", what, &mut wrong);
    for pole in poles {
        let images = answer
            .images_of_vertex(pole)
            .expect("this model made the key");
        if !images.is_empty() {
            wrong.push(format!(
                "{what}: the ball's pole {pole:?} should be deleted, got {images:?}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "a ball on its side keeps nothing of its poles and seam:\n{}",
        wrong.join("\n")
    );
}

/// A bar inside a block, cut out of it or kept as what the two share: no
/// rule cuts the bar's wall, so the answer writes it as a band between its two
/// rims, with no edge where the bar's seam ran.
#[test]
fn a_bar_kept_whole_keeps_its_rims_and_their_corners_but_not_its_seam() {
    let bottom = [6.0, 5.0, 1.0];
    let top = [6.0, 5.0, 3.0];
    let mut wrong: Vec<String> = Vec::new();
    for (rule, bar_first) in [
        (Rule::Difference, false),
        (Rule::Intersection, false),
        (Rule::Intersection, true),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = a_box(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
        let bar = model
            .add_round_bar(
                1.0,
                2.0,
                &Isometry::translation(Vector::new(5.0, 5.0, 1.0)).expect("a whole-number shift"),
            )
            .expect("a bar of radius 1 and height 2");
        let seam = the_edge_between(&model, bar, [bottom, top]);
        let bottom_rim = the_edge_between(&model, bar, [bottom, bottom]);
        let top_rim = the_edge_between(&model, bar, [top, top]);
        let bottom_corner = the_vertex_at(&model, bar, bottom);
        let top_corner = the_vertex_at(&model, bar, top);

        let [first, second] = if bar_first {
            [bar, block]
        } else {
            [block, bar]
        };
        let what = format!("a bar inside a block, {rule:?}, bar first {bar_first}");
        let answer = model
            .fuse(
                &[first],
                &[second],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} refused: {why}"));

        check_deleted(&model, &answer, seam, "the bar's seam", &what, &mut wrong);
        check_survives(
            &model,
            &answer,
            bottom_rim,
            [bottom, bottom],
            "the bar's bottom rim",
            &what,
            &mut wrong,
        );
        check_survives(
            &model,
            &answer,
            top_rim,
            [top, top],
            "the bar's top rim",
            &what,
            &mut wrong,
        );
        check_corner_survives(
            &model,
            &answer,
            bottom_corner,
            bottom,
            "the bar's bottom corner",
            &what,
            &mut wrong,
        );
        check_corner_survives(
            &model,
            &answer,
            top_corner,
            top,
            "the bar's top corner",
            &what,
            &mut wrong,
        );
        let along_the_seam: Vec<EdgeKey> = answer
            .solids()
            .iter()
            .flat_map(|&solid| located_edges_of(&model, solid))
            .filter(|&(_, ends)| ends == [bottom, top])
            .map(|(edge, _)| edge)
            .collect();
        if !along_the_seam.is_empty() {
            wrong.push(format!(
                "{what}: the answer has edges {along_the_seam:?} where the bar's seam ran, \
                 which its history should name"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "a bar kept whole keeps its rims and their corners, and its seam has nowhere to go:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn edge_history_of_a_block_standing_flush_does_not_depend_on_argument_order() {
    let mut wrong: Vec<String> = Vec::new();
    for swapped in [false, true] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let lower = a_box(&mut model, [0.0, 0.0, 0.0], [4.0, 4.0, 4.0]);
        let upper = a_box(&mut model, [0.0, 0.0, 4.0], [2.0, 4.0, 2.0]);

        let lower_x0 = the_edge_at(&model, lower, |x, _, z| x == 0.0 && z == 4.0);
        let lower_x4 = the_edge_at(&model, lower, |x, _, z| x == 4.0 && z == 4.0);
        let lower_y0 = the_edge_at(&model, lower, |_, y, z| y == 0.0 && z == 4.0);
        let lower_y4 = the_edge_at(&model, lower, |_, y, z| y == 4.0 && z == 4.0);
        let upper_x0 = the_edge_at(&model, upper, |x, _, z| x == 0.0 && z == 4.0);
        let upper_x2 = the_edge_at(&model, upper, |x, _, z| x == 2.0 && z == 4.0);
        let upper_y0 = the_edge_at(&model, upper, |_, y, z| y == 0.0 && z == 4.0);
        let upper_y4 = the_edge_at(&model, upper, |_, y, z| y == 4.0 && z == 4.0);

        let [first, second] = if swapped {
            [upper, lower]
        } else {
            [lower, upper]
        };
        let what = format!("a block standing flush, swapped {swapped}");
        let answer = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} refused: {why}"));

        check_deleted(
            &model,
            &answer,
            lower_x0,
            "the lower block's edge at x = 0",
            &what,
            &mut wrong,
        );
        check_deleted(
            &model,
            &answer,
            upper_x0,
            "the upper block's edge at x = 0",
            &what,
            &mut wrong,
        );
        check_deleted(
            &model,
            &answer,
            upper_y0,
            "the upper block's edge at y = 0",
            &what,
            &mut wrong,
        );
        check_deleted(
            &model,
            &answer,
            upper_y4,
            "the upper block's edge at y = 4",
            &what,
            &mut wrong,
        );

        check_survives(
            &model,
            &answer,
            lower_x4,
            [[4.0, 0.0, 4.0], [4.0, 4.0, 4.0]],
            "the lower block's edge at x = 4",
            &what,
            &mut wrong,
        );
        check_survives(
            &model,
            &answer,
            upper_x2,
            [[2.0, 0.0, 4.0], [2.0, 4.0, 4.0]],
            "the upper block's edge at x = 2",
            &what,
            &mut wrong,
        );
        check_survives(
            &model,
            &answer,
            lower_y0,
            [[2.0, 0.0, 4.0], [4.0, 0.0, 4.0]],
            "the lower block's edge at y = 0, cut short at x = 2",
            &what,
            &mut wrong,
        );
        check_survives(
            &model,
            &answer,
            lower_y4,
            [[2.0, 4.0, 4.0], [4.0, 4.0, 4.0]],
            "the lower block's edge at y = 4, cut short at x = 2",
            &what,
            &mut wrong,
        );
    }
    assert!(
        wrong.is_empty(),
        "an edge's history must come out the same whichever block is handed to fuse first:\n{}",
        wrong.join("\n")
    );
}

/// An edge deleted outright has no image, whichever block came first.
fn check_deleted(
    model: &Model,
    answer: &Fused,
    edge: EdgeKey,
    name: &str,
    what: &str,
    wrong: &mut Vec<String>,
) {
    let images = answer
        .images_of_edge(edge)
        .expect("this model made the key");
    if !images.is_empty() {
        let found: Vec<[[f64; 3]; 2]> = images
            .iter()
            .map(|&image| locate(model, answer.solids(), image))
            .collect();
        wrong.push(format!(
            "{what}: {name} should be deleted, got images at {found:?}"
        ));
    }
}

/// An edge's one image is at exactly `wanted`, whichever block came first.
fn check_survives(
    model: &Model,
    answer: &Fused,
    edge: EdgeKey,
    wanted: [[f64; 3]; 2],
    name: &str,
    what: &str,
    wrong: &mut Vec<String>,
) {
    let wanted = sorted_ends(wanted);
    let images = answer
        .images_of_edge(edge)
        .expect("this model made the key");
    let found: Vec<[[f64; 3]; 2]> = images
        .iter()
        .map(|&image| locate(model, answer.solids(), image))
        .collect();
    if found != vec![wanted] {
        wrong.push(format!(
            "{what}: {name} should be at {wanted:?}, got {found:?}"
        ));
    }
}

/// A corner's one image is at exactly `wanted`.
fn check_corner_survives(
    model: &Model,
    answer: &Fused,
    corner: VertexKey,
    wanted: [f64; 3],
    name: &str,
    what: &str,
    wrong: &mut Vec<String>,
) {
    let found: Vec<[f64; 3]> = answer
        .images_of_vertex(corner)
        .expect("this model made the key")
        .iter()
        .map(|&image| locate_corner(model, answer.solids(), image))
        .collect();
    if found != vec![wanted] {
        wrong.push(format!(
            "{what}: {name} should be at {wanted:?}, got {found:?}"
        ));
    }
}

/// Where `corner` is, among the corners of `solids`, for [`locate`]'s reason.
fn locate_corner(model: &Model, solids: &[SolidKey], corner: VertexKey) -> [f64; 3] {
    for &solid in solids {
        if let Some(at) = located_corners_of(model, solid)
            .into_iter()
            .find_map(|(key, at)| (key == corner).then_some(at))
        {
            return at;
        }
    }
    panic!("no solid among {solids:?} has the corner {corner:?}")
}

/// Where `edge`'s two ends are, among the edges of `solids` — not
/// `Model::edge`, which answers in the model's own frame and misses the
/// placement a shell was built with (`Model::add_extrusion`'s doc comment).
fn locate(model: &Model, solids: &[SolidKey], edge: EdgeKey) -> [[f64; 3]; 2] {
    for &solid in solids {
        if let Some(ends) = located_edges_of(model, solid)
            .into_iter()
            .find_map(|(key, ends)| (key == edge).then_some(ends))
        {
            return ends;
        }
    }
    panic!("no solid among {solids:?} has the edge {edge:?}")
}

fn sorted_ends(mut ends: [[f64; 3]; 2]) -> [[f64; 3]; 2] {
    ends.sort_by(|a, b| {
        a[0].total_cmp(&b[0])
            .then(a[1].total_cmp(&b[1]))
            .then(a[2].total_cmp(&b[2]))
    });
    ends
}

/// The one straight edge of `solid` whose both ends satisfy `at`.
fn the_edge_at(model: &Model, solid: SolidKey, at: impl Fn(f64, f64, f64) -> bool) -> EdgeKey {
    let matches: Vec<EdgeKey> = located_edges_of(model, solid)
        .into_iter()
        .filter(|(_, [one, other])| at(one[0], one[1], one[2]) && at(other[0], other[1], other[2]))
        .map(|(edge, _)| edge)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one such edge, got {matches:?}"
    );
    matches[0]
}

/// The one edge of `solid` running from one of `ends` to the other; a closed
/// edge starts and ends at the same place.
fn the_edge_between(model: &Model, solid: SolidKey, ends: [[f64; 3]; 2]) -> EdgeKey {
    let ends = sorted_ends(ends);
    let matches: Vec<EdgeKey> = located_edges_of(model, solid)
        .into_iter()
        .filter(|&(_, there)| there == ends)
        .map(|(edge, _)| edge)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one edge between {ends:?}, got {matches:?}"
    );
    matches[0]
}

/// The one corner of `solid` at exactly `at`.
fn the_vertex_at(model: &Model, solid: SolidKey, at: [f64; 3]) -> VertexKey {
    let matches: Vec<VertexKey> = located_corners_of(model, solid)
        .into_iter()
        .filter(|&(_, there)| there == at)
        .map(|(corner, _)| corner)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one corner at {at:?}, got {matches:?}"
    );
    matches[0]
}

/// Every corner of a solid, once each, moved into place, the way
/// [`located_edges_of`] reads the edges.
fn located_corners_of(model: &Model, solid: SolidKey) -> Vec<(VertexKey, [f64; 3])> {
    let mut corners: Vec<(VertexKey, [f64; 3])> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                for corner in [coedge.start(), coedge.end()] {
                    let key = corner.key();
                    if corners.iter().any(|&(seen, _)| seen == key) {
                        continue;
                    }
                    let at = corner.point();
                    corners.push((key, [at.x(), at.y(), at.z()]));
                }
            }
        }
    }
    corners
}

/// Every edge of a solid, once each, with its two ends already moved into
/// place — a solid built by `Model::add_extrusion` carries its placement on
/// the shell, composed on the way out through this very walk
/// (`Model::add_extrusion`'s doc comment), and nowhere else.
fn located_edges_of(model: &Model, solid: SolidKey) -> Vec<(EdgeKey, [[f64; 3]; 2])> {
    let mut edges: Vec<(EdgeKey, [[f64; 3]; 2])> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                let key = edge.key();
                if edges.iter().any(|&(seen, _)| seen == key) {
                    continue;
                }
                let one = edge.start().point();
                let other = edge.end().point();
                edges.push((
                    key,
                    sorted_ends([
                        [one.x(), one.y(), one.z()],
                        [other.x(), other.y(), other.z()],
                    ]),
                ));
            }
        }
    }
    edges
}

/// A box with its low corner at `[x, y, z]`, sized `[dx, dy, dz]`.
fn a_box(model: &mut Model, [x, y, z]: [f64; 3], [dx, dy, dz]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [dx, 0.0], [dx, dy], [0.0, dy]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            dz,
            &Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift"),
        )
        .expect("building a box needs no decision")
}
