use rcad_predicates::{Decision, Point, Policy, ProvenanceLog, orient3d_tolerantly};

use crate::{Error, Result};

/// Which way three points in a row turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Turn {
    /// Anticlockwise.
    Left,
    /// All three on one straight line.
    Straight,
    /// Clockwise.
    Right,
}

/// Which way `a`, `b`, `c` turn, decided exactly.
///
/// There is no `orient2d` in `rcad-predicates`, and there does not need to be:
/// a flat determinant is a three-dimensional one with a unit step off the
/// plane. Putting `a` at height one and everything else at height zero leaves
/// exactly the two-by-two determinant behind, so the exact cascade decides it
/// for free.
///
/// # Errors
///
/// [`Error::Undecidable`] when the cascade cannot settle the sign. For plain
/// `f64` inputs that only happens when a product overflows, which is what the
/// dynamic-range check exists to catch (decisions.md §3.5).
pub(crate) fn turn_of(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Turn> {
    let decision = orient3d_tolerantly(
        Point::from([b[0], b[1], 0.0]),
        Point::from([c[0], c[1], 0.0]),
        Point::from([a[0], a[1], 1.0]),
        Point::from([a[0], a[1], 0.0]),
        policy,
        log,
    );
    match decision {
        Decision::Pos => Ok(Turn::Left),
        Decision::Zero => Ok(Turn::Straight),
        Decision::Neg => Ok(Turn::Right),
        Decision::Undecidable { .. } => Err(Error::Undecidable {
            records: core::mem::take(log),
        }),
        // `Decision` grows a variant at every milestone. Whatever the new one
        // means, guessing is worse than stopping.
        other => panic!("orient3d came back with a decision nothing here knows: {other:?}"),
    }
}

/// Whether the two closed segments share any point at all.
///
/// # Errors
///
/// [`Error::Undecidable`] if any of the four turns cannot be decided.
pub(crate) fn segments_meet(
    first: [[f64; 2]; 2],
    second: [[f64; 2]; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let [p, q] = first;
    let [r, s] = second;

    let p_side = turn_of(r, s, p, policy, log)?;
    let q_side = turn_of(r, s, q, policy, log)?;
    let r_side = turn_of(p, q, r, policy, log)?;
    let s_side = turn_of(p, q, s, policy, log)?;

    let straddles_second = p_side != q_side && p_side != Turn::Straight && q_side != Turn::Straight;
    let straddles_first = r_side != s_side && r_side != Turn::Straight && s_side != Turn::Straight;
    if straddles_second && straddles_first {
        return Ok(true);
    }

    Ok((p_side == Turn::Straight && point_is_between(p, r, s))
        || (q_side == Turn::Straight && point_is_between(q, r, s))
        || (r_side == Turn::Straight && point_is_between(r, p, q))
        || (s_side == Turn::Straight && point_is_between(s, p, q)))
}

/// Whether `point` is inside a ring, on one of its edges, or outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Where {
    Inside,
    OnEdge,
    Outside,
}

/// Where `point` sits relative to the closed ring `ring`.
///
/// Counts how many times the ring winds round the point, which is right for a
/// ring that runs either way and does not care whether the ring is convex.
///
/// # Errors
///
/// [`Error::Undecidable`] if any turn cannot be decided.
pub(crate) fn where_point_sits(
    point: [f64; 2],
    ring: &[[f64; 2]],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Where> {
    let mut winding = 0i32;
    for (index, &start) in ring.iter().enumerate() {
        let end = ring[(index + 1) % ring.len()];
        let side = turn_of(start, end, point, policy, log)?;
        if side == Turn::Straight && point_is_between(point, start, end) {
            return Ok(Where::OnEdge);
        }
        if start[1] <= point[1] {
            if end[1] > point[1] && side == Turn::Left {
                winding += 1;
            }
        } else if end[1] <= point[1] && side == Turn::Right {
            winding -= 1;
        }
    }
    Ok(if winding == 0 {
        Where::Outside
    } else {
        Where::Inside
    })
}

/// Whether the ring runs anticlockwise.
///
/// Read off the corner nobody can argue about: the lowest point, and the
/// leftmost of those. A simple ring always turns the same way as the ring does
/// at that corner.
///
/// # Errors
///
/// [`Error::ProfileEnclosesNoArea`] if that corner is straight, which a simple
/// ring cannot manage. [`Error::Undecidable`] if the turn cannot be decided.
pub(crate) fn ring_runs_anticlockwise(
    ring: &[[f64; 2]],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let count = ring.len();
    let mut lowest = 0;
    for (index, point) in ring.iter().enumerate() {
        let best = ring[lowest];
        if point[1] < best[1] || (point[1] == best[1] && point[0] < best[0]) {
            lowest = index;
        }
    }
    let before = ring[(lowest + count - 1) % count];
    let after = ring[(lowest + 1) % count];
    match turn_of(before, ring[lowest], after, policy, log)? {
        Turn::Left => Ok(true),
        Turn::Right => Ok(false),
        Turn::Straight => Err(Error::ProfileEnclosesNoArea),
    }
}

/// Whether `point`, already known to be on the line through `start` and `end`,
/// is also between them.
pub(crate) fn point_is_between(point: [f64; 2], start: [f64; 2], end: [f64; 2]) -> bool {
    (0..2).all(|axis| {
        point[axis] >= start[axis].min(end[axis]) && point[axis] <= start[axis].max(end[axis])
    })
}
