use crate::Interval;

/// A point the predicates can work with.
///
/// The split matters: exact coordinates can be pushed through level 3 of the
/// cascade and always get an answer, while a constructed point cannot. Exact
/// arithmetic on a coordinate we only know approximately gives an exact answer
/// to the wrong question (decisions.md §2.5).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Point {
    /// Coordinates read straight out of a model, exact as written.
    Exact([f64; 3]),
    /// A point produced by an intersection. All we know is that it lies inside
    /// this box.
    Constructed([Interval; 3]),
}

impl Point {
    /// The coordinates, if they are exact. `None` for a constructed point.
    pub fn exact_coordinates(self) -> Option<[f64; 3]> {
        match self {
            Point::Exact(coordinates) => Some(coordinates),
            Point::Constructed(_) => None,
        }
    }

    /// A box the point is somewhere inside. For an exact point every side has
    /// zero width.
    pub fn enclosure(self) -> [Interval; 3] {
        match self {
            Point::Exact([x, y, z]) => [
                Interval::exactly(x),
                Interval::exactly(y),
                Interval::exactly(z),
            ],
            Point::Constructed(box_) => box_,
        }
    }

    /// The longest side of [`Point::enclosure`]. Zero for an exact point.
    pub fn uncertainty(self) -> f64 {
        self.enclosure()
            .iter()
            .map(|side| side.width())
            .fold(0.0, f64::max)
    }
}

impl From<[f64; 3]> for Point {
    fn from(coordinates: [f64; 3]) -> Self {
        Point::Exact(coordinates)
    }
}

#[cfg(test)]
mod tests {
    use super::Point;
    use crate::Interval;

    #[test]
    fn an_exact_point_has_no_uncertainty() {
        let point = Point::from([1.0, 2.0, 3.0]);
        assert_eq!(point.exact_coordinates(), Some([1.0, 2.0, 3.0]));
        assert_eq!(point.uncertainty(), 0.0);
    }

    #[test]
    fn a_constructed_point_reports_its_widest_side() {
        let point = Point::Constructed([
            Interval::exactly(1.0),
            Interval::new(2.0, 2.5).unwrap(),
            Interval::new(3.0, 3.25).unwrap(),
        ]);
        assert_eq!(point.exact_coordinates(), None);
        assert_eq!(point.uncertainty(), 0.5);
    }
}
