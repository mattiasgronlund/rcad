use crate::{Decision, Point};

/// A surface that can say which side of itself a point is on.
///
/// `rcad-geom` implements this for `Sphere` and `Cylinder`. The quadric types
/// live there, so the predicate does too; only the contract lives here
/// (decisions.md §7.2). That is what keeps `rcad-geom` free of any dependency
/// on topology.
///
/// One method, because that is all an inside-outside test is. An implementation
/// evaluates its own quadric through the cascade and returns what it decided.
pub trait SideOfSurface {
    /// [`Decision::Neg`] inside, [`Decision::Pos`] outside,
    /// [`Decision::Zero`] on the surface, and [`Decision::Undecidable`] when
    /// the cascade could not tell — which a constructed `point` can genuinely
    /// cause.
    fn side_of(&self, point: Point) -> Decision;
}

#[cfg(test)]
mod tests {
    use super::SideOfSurface;
    use crate::expansion::{
        expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_diff, two_product,
    };
    use crate::{Decision, Filtered, Interval, Level, Point, Predicate, decide};

    /// A sphere, standing in for the quadrics `rcad-geom` will bring. Here to
    /// show a second predicate joining the cascade by supplying three
    /// evaluations and nothing else.
    #[derive(Clone, Copy)]
    struct Sphere {
        centre: [f64; 3],
        radius: f64,
    }

    impl SideOfSurface for Sphere {
        fn side_of(&self, point: Point) -> Decision {
            decide(&HowFarOut {
                sphere: *self,
                point,
            })
            .0
        }
    }

    /// `(p - centre)·(p - centre) - radius²`: below zero inside, above outside.
    struct HowFarOut {
        sphere: Sphere,
        point: Point,
    }

    impl Predicate for HowFarOut {
        fn name(&self) -> &'static str {
            "side of sphere"
        }

        fn on_floats(&self) -> Option<Filtered> {
            let point = self.point.exact_coordinates()?;
            let squares = [0, 1, 2].map(|axis| {
                let gap = point[axis] - self.sphere.centre[axis];
                gap * gap
            });
            let radius_squared = self.sphere.radius * self.sphere.radius;

            // Deliberately looser than the six roundings this actually takes.
            // Too large a bound only costs level 1 hits; too small a one costs
            // correctness.
            const ERROR_BOUND: f64 = {
                let epsilon = f64::EPSILON / 2.0;
                (8.0 + 64.0 * epsilon) * epsilon
            };
            let permanent = squares[0] + squares[1] + squares[2] + radius_squared;

            Some(Filtered {
                value: squares[0] + squares[1] + squares[2] - radius_squared,
                error_bound: ERROR_BOUND * permanent,
            })
        }

        fn on_intervals(&self) -> Interval {
            let point = self.point.enclosure();
            let squared_gap = |axis: usize| {
                let gap = point[axis] - Interval::exactly(self.sphere.centre[axis]);
                gap * gap
            };
            let radius = Interval::exactly(self.sphere.radius);
            squared_gap(0) + squared_gap(1) + squared_gap(2) - radius * radius
        }

        fn exactly(&self) -> Option<Decision> {
            let point = self.point.exact_coordinates()?;
            let mut total = [0.0f64; 32];
            let mut total_len = 0;
            let mut merged = [0.0f64; 32];

            for (coordinate, centre) in point.into_iter().zip(self.sphere.centre) {
                let mut squared = [0.0f64; 8];
                let len = squared_difference(coordinate, centre, &mut squared)?;
                total_len = expansion_sum(&total[..total_len], &squared[..len], &mut merged);
                core::mem::swap(&mut total, &mut merged);
            }

            if !product_is_exact(self.sphere.radius, self.sphere.radius) {
                return None;
            }
            let (radius_squared, roundoff) = two_product(self.sphere.radius, self.sphere.radius);
            total_len = expansion_sum(
                &total[..total_len],
                &[-roundoff, -radius_squared],
                &mut merged,
            );

            match expansion_sign(&merged[..total_len]) {
                Decision::Neg => Some(Decision::Neg),
                Decision::Zero => Some(Decision::Zero),
                Decision::Pos => Some(Decision::Pos),
                Decision::Undecidable { .. } => None,
            }
        }
    }

    /// `(a - b)²`, exactly. The difference is exact in two entries, and
    /// squaring those two is two scalings and a sum.
    fn squared_difference(a: f64, b: f64, out: &mut [f64; 8]) -> Option<usize> {
        let (high, low) = two_diff(a, b);
        let difference = [low, high];
        if !difference
            .iter()
            .all(|&entry| product_is_exact(entry, low) && product_is_exact(entry, high))
        {
            return None;
        }

        let mut by_low = [0.0; 4];
        let by_low_len = scale_expansion(&difference, low, &mut by_low);
        let mut by_high = [0.0; 4];
        let by_high_len = scale_expansion(&difference, high, &mut by_high);
        Some(expansion_sum(
            &by_low[..by_low_len],
            &by_high[..by_high_len],
            out,
        ))
    }

    const UNIT_SPHERE: Sphere = Sphere {
        centre: [0.0, 0.0, 0.0],
        radius: 1.0,
    };

    #[test]
    fn a_point_outside_a_sphere_is_positive_and_one_inside_is_negative() {
        assert_eq!(
            UNIT_SPHERE.side_of(Point::from([2.0, 0.0, 0.0])),
            Decision::Pos
        );
        assert_eq!(
            UNIT_SPHERE.side_of(Point::from([0.25, 0.25, 0.25])),
            Decision::Neg
        );
    }

    /// The legs and hypotenuse of a Pythagorean triple built from m = 12345,
    /// n = 6789. Every coordinate is a whole number, so the point is exactly on
    /// the sphere — but the squares run past 53 bits, so only exact arithmetic
    /// can see it.
    #[test]
    fn a_point_exactly_on_a_big_sphere_needs_exact_arithmetic() {
        let sphere = Sphere {
            centre: [0.0, 0.0, 0.0],
            radius: 198_489_546.0,
        };
        let on_it = Point::from([106_308_504.0, 167_620_410.0, 0.0]);

        let (decision, level) = decide(&HowFarOut {
            sphere,
            point: on_it,
        });
        assert_eq!(decision, Decision::Zero);
        assert_eq!(level, Level::Exact);
        assert_eq!(sphere.side_of(on_it), Decision::Zero);
    }

    #[test]
    fn a_constructed_point_on_the_surface_cannot_be_placed() {
        // The box around (1/3, 1/3, 1/3), on a sphere of radius sqrt(1/3)
        // through it — except sqrt(1/3) is not an f64 either, so use the
        // sphere x² + y² + z² = 1/3 by way of a radius that squares to it.
        let third = crate::meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();
        let sphere = Sphere {
            centre: [0.0, 0.0, 0.0],
            radius: (1.0f64 / 3.0).sqrt(),
        };

        match sphere.side_of(third) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("neither the box nor the radius is exact enough to place this")
            }
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
        }
    }
}
