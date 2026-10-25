//! What can go wrong when building geometry.

/// Everything this crate refuses to build, and why.
///
/// Almost all of these are bad caller input: numbers that cannot describe the
/// shape they are meant to describe. A broken invariant *inside* an
/// already-valid value is not here — that is a bug, and it panics
/// (`docs/decisions.md` §4.1).
///
/// [`Error::Undecidable`] is the exception, and it is new. Until M3 this crate
/// held nothing but bad input, because it had no *operation* that could give up:
/// a predicate that cannot decide returns a `Decision`, which is a result and
/// not a failure. Naming where two surfaces cross is an operation, it is a chain
/// of sign decisions, and a chain with a link missing has nothing to hand back
/// (§24.10).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// A coordinate was infinite or not a number.
    #[error("the {what} has a coordinate that is not a finite number")]
    NotFinite {
        /// Which piece of input it was.
        what: &'static str,
    },

    /// A vector that has to say which way it points has no length to say it
    /// with.
    #[error("the {what} has no length, so it does not point anywhere")]
    ZeroLength {
        /// Which piece of input it was.
        what: &'static str,
    },

    /// Quarter turns were asked for about an axis that does not run exactly
    /// along `x`, `y` or `z`.
    ///
    /// [`Isometry::quarter_turns`](crate::Isometry::quarter_turns) promises a
    /// turn that rounds nothing, and only a turn about one of those three moves
    /// each coordinate onto another. About any other axis a quarter turn is
    /// [`Isometry::rotation`](crate::Isometry::rotation)'s to build, and that
    /// rounds.
    #[error(
        "quarter turns are only said exactly about an axis along x, y or z, and this one points \
         along {direction:?}"
    )]
    QuarterTurnsNeedAnAxisAlongXYOrZ {
        /// The direction of the axis that was asked about.
        direction: crate::Vector,
    },

    /// Three points that were meant to give a plane all sit on one line.
    #[error("the three points are all on one line, so they do not give a plane")]
    PointsAreInALine,

    /// A sphere or cylinder was asked for with a radius that is not a size.
    #[error("the radius must be greater than zero, but it was {radius}")]
    RadiusNotPositive {
        /// What the caller asked for.
        radius: f64,
    },

    /// A cone was asked for with a spread that is not a spread.
    #[error("a cone's slope must be greater than zero, but it was {slope}")]
    SlopeNotPositive {
        /// What the caller asked for.
        slope: f64,
    },

    /// A cone was named by a half-angle that is not one.
    ///
    /// At zero a cone is a line and at a right angle it is a plane, so the open
    /// range between them is the whole of what names a cone.
    #[error("a cone's half-angle must be between zero and a right angle, but it was {half_angle}")]
    HalfAngleIsNotACone {
        /// What the caller asked for.
        half_angle: f64,
    },

    /// A torus was asked for with a tube as wide as its ring, or wider.
    ///
    /// Such a surface runs through itself, and — the reason this is refused at
    /// the door rather than noted in a doc comment — the quartic
    /// [`Torus`](crate::Torus)'s sign predicate reads **stops meaning inside and
    /// outside**. `a_tube_wider_than_its_ring_would_certify_the_wrong_side`
    /// writes out the quartic by hand for a point plainly inside such a tube and
    /// shows it come back positive, which every level of the cascade would
    /// certify as outside. A confident wrong answer is what §4.1 ranks below a
    /// crash.
    ///
    /// It is caller input rather than a broken invariant: two radii arrive from
    /// a stranger's bytes through `Deserialize` as readily as from a call.
    #[error(
        "a torus's tube radius {tube_radius} must be less than its ring radius {ring_radius}, or \
         the surface runs through itself"
    )]
    ATorusThatRunsThroughItself {
        /// How far the middle of the tube runs from the axis.
        ring_radius: f64,
        /// How thick the tube is.
        tube_radius: f64,
    },

    /// A mesh was asked to stray a distance that is not a distance.
    #[error("a deflection must be greater than zero, but it was {length}")]
    DeflectionNotPositive {
        /// What the caller asked for.
        length: f64,
    },

    /// The two ends of a segment are the same point.
    #[error("the two ends of the segment are the same point")]
    SegmentHasNoLength,

    /// A parameter fell outside the stretch of curve it was meant to name.
    #[error("the parameter {parameter} is not between 0 and the segment length {length}")]
    ParameterOutsideSegment {
        /// What the caller asked for.
        parameter: f64,
        /// How far along the segment it could have gone.
        length: f64,
    },

    /// A model was declared with a size that is not a size.
    #[error("the nominal size must be greater than zero, but it was {nominal_size}")]
    NominalSizeNotPositive {
        /// What the caller asked for.
        nominal_size: f64,
    },

    /// The smallest feature in a model was given as zero or less.
    #[error("the smallest feature size must be greater than zero, but it was {size}")]
    FeatureSizeNotPositive {
        /// What the caller asked for.
        size: f64,
    },

    /// The model spans too many orders of magnitude to be resolved in `f64`.
    #[error(
        "the model spans {ratio:e} times its smallest feature, past the {limit:e} that f64 can carry",
        limit = crate::DynamicRange::LARGEST_SUPPORTED_RATIO
    )]
    DynamicRangeTooWide {
        /// Largest coordinate magnitude divided by smallest feature size.
        ratio: f64,
    },

    /// A spline was asked for at a degree this kernel does not carry.
    #[error(
        "the degree must be between 1 and {limit}, but it was {degree}",
        limit = crate::MAX_DEGREE
    )]
    DegreeNotSupported {
        /// What the caller asked for.
        degree: usize,
    },

    /// A knot vector too short to be clamped at both ends.
    ///
    /// A degree `p` spline needs `p + 1` knots at each end before anything else
    /// can be said about it.
    #[error("{knots} knots is too few for a degree {degree} spline")]
    TooFewKnots {
        /// The degree that was asked for.
        degree: usize,
        /// How many knots came with it.
        knots: usize,
    },

    /// A knot is smaller than the one before it.
    #[error("the knot at position {at} is smaller than the one before it")]
    KnotsGoBackwards {
        /// Where the drop is.
        at: usize,
    },

    /// The first and last knot are so far apart that subtracting one from the
    /// other overflows.
    ///
    /// Every difference the de Boor recursion forms is between two of the
    /// stored knots, so none of them is wider than this one. A range that
    /// overflows makes every de Boor coefficient come out as zero, and the
    /// published evaluation bound certifies the wrong point that follows,
    /// because its derivation assumes the subtractions did not overflow.
    #[error("the knots run from {lo} to {hi}, which is too far apart to subtract in f64")]
    KnotsTooFarApart {
        /// The first knot.
        lo: f64,
        /// The last knot.
        hi: f64,
    },

    /// The first or last knot is not repeated enough times to pin the curve to
    /// its end control point.
    #[error(
        "the first and last knot must each be repeated one more time than the degree; \
         periodic knot vectors first arrive with STEP import at M5"
    )]
    KnotVectorNotClamped,

    /// An interior knot appears more often than the degree allows, which would
    /// leave a control point with no basis function of its own.
    #[error("the interior knot {knot} appears {times} times, more than the degree {degree}")]
    KnotRepeatedTooOften {
        /// The knot value that repeats.
        knot: f64,
        /// How many times it appears in the whole vector.
        times: usize,
        /// The degree it was measured against.
        degree: usize,
    },

    /// Every span between the ends has zero width, so the knot vector names no
    /// stretch of parameter at all.
    #[error("every span between the first and last knot has zero width")]
    KnotVectorHasNoSpan,

    /// The control net does not have the number of points the knot vector asks
    /// for.
    #[error("the knot vector asks for {expected} control points, but {given} were given")]
    WrongNumberOfControlPoints {
        /// How many the knot vector asks for.
        expected: usize,
        /// How many the caller handed over.
        given: usize,
    },

    /// A weight that cannot be divided back out.
    #[error("every weight must be a finite number greater than zero, but one was {weight}")]
    WeightNotPositive {
        /// The weight that was refused.
        weight: f64,
    },

    /// A weight was positive but down among the subnormal numbers, where a
    /// blend of two of them can round away to nothing.
    ///
    /// A rational spline depends on its weights only through their ratios, so
    /// multiplying every weight by a power of two gives back the identical
    /// curve, exactly and with no bits lost. That makes refusing lose no curve
    /// **only while the weights span less than `f64`'s normal range**:
    /// weights of `5e-324` and `1e300` need `2⁵²` to bring the smaller up to a
    /// normal number, and `1e300 · 2⁵²` is past `f64::MAX`. Whether the curve
    /// such weights name can be held some other way is not shown, so the class
    /// is **not known** (§437.6).
    #[error(
        "the weight {weight:e} is too small to blend; scale every weight up by a power of two, \
         but only if the largest weight still has room to grow"
    )]
    WeightTooSmallToBlend {
        /// The weight that was refused.
        weight: f64,
    },

    /// A parameter fell outside the stretch of parameter a spline is defined
    /// over.
    #[error("the parameter {parameter} is not between {lo} and {hi}")]
    ParameterOutsideDomain {
        /// What the caller asked for.
        parameter: f64,
        /// The bottom of the domain.
        lo: f64,
        /// The top of the domain.
        hi: f64,
    },

    /// A knot was asked to be inserted where it already repeats as often as the
    /// degree allows.
    ///
    /// One more copy would leave a control point with no basis function of its
    /// own, which is the same thing [`Error::KnotRepeatedTooOften`] refuses at
    /// construction. It is a separate variant because the caller here did not
    /// hand over a bad knot vector — it handed over a good one and asked for a
    /// refinement of it that does not exist.
    #[error(
        "the knot {knot} already appears {times} times, and a degree {degree} spline allows no more"
    )]
    CannotRepeatKnotAgain {
        /// The knot the caller asked to insert.
        knot: f64,
        /// How many times it already appears.
        times: usize,
        /// The degree it was measured against.
        degree: usize,
    },

    /// A curve was asked to be split at one end of its own domain, where one of
    /// the two halves would be no curve at all.
    #[error(
        "splitting at {parameter} leaves nothing on one side; the domain runs from {lo} to {hi}"
    )]
    NothingOnOneSideOfTheSplit {
        /// What the caller asked for.
        parameter: f64,
        /// The bottom of the domain.
        lo: f64,
        /// The top of the domain.
        hi: f64,
    },

    /// Two surfaces tangent at a point of their crossing, so what they cross in
    /// is one curve that runs through itself there.
    ///
    /// A ball resting on the inside of a pipe from the far side, or a thin pipe
    /// resting on the inside of a wider one: each crosses in a quartic whose two
    /// loops touch where the two surfaces are tangent, and no half-space picks
    /// one of them without a plane through exactly that point. Two pipes of one
    /// width whose axes meet also cross in a curve that runs through itself, but
    /// theirs is two ellipses that need no marcher, and `intersect` names them
    /// as four halves (`docs/decisions.md` §31.9).
    ///
    /// **`intersect` gives this for neither.** It names both as two loops,
    /// each picked out by one side of a flat surface exactly through the
    /// touching point: through the wide pipe's axis for two pipes (Q2628), and
    /// through the ball's centre square to the pipe's axis for the ball (Q2649).
    /// The fuse's arrangement in `rcad-topo` gives this for two loops of two
    /// pipes that it can prove neither apart nor meeting, since it has nowhere
    /// to put the node where they meet.
    ///
    /// **A gap** (§16.10, §31.9). [`EdgeRun`](crate::EdgeRun) *can* say this:
    /// the two loops of a pinch are the two sides of one half-space through the
    /// tangent point, and for the ball and pipe in
    /// `what_a_geometry_refusal_says_about_its_answer.rs` the tangent point
    /// `(-2, 0, 0)` is one `f64` holds exactly.
    ///
    /// Where a crossing runs through itself the two surfaces are **tangent**,
    /// so the crossing has no direction there and the marcher has nothing to
    /// follow from that point.
    /// `marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow`
    /// is the demonstration. The thin pipe's two loops, named and let through
    /// the arrangement without a node, were walked by five of the six fuses of
    /// a bar resting inside a wider one, and what was wrong with those answers
    /// was that no vertex was where the loops meet.
    ///
    /// The family of fixes: a node where the loops meet, on both walls, and a
    /// marcher that walks each loop between that node and one away from it.
    #[error(
        "these two surfaces are tangent along where they cross, so what they cross in is one \
         curve that runs through itself"
    )]
    ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo,

    /// A crossing came in two pieces and no half-space could be written down
    /// that picks one of them out and holds along the whole of it.
    ///
    /// **Not known** (§437.6), though the two pieces are there and they are
    /// tellable apart. What runs out is `f64`'s supply of unit vectors exactly
    /// square to a given direction, and that is sometimes a picker not found
    /// and sometimes no picker at all. `two_pipes_of_one_width_on_crossing_axes.rs`
    /// shows both: bores on `(3, 4, 0)` and `(4, 0, 3)` have no flat surface
    /// written in `f64` that picks their halves, which is a limit of today's
    /// [`HalfSpace`](crate::HalfSpace), while the same file's count of named
    /// pairs, raised from 32 to 1096 of 1224 by searching harder, shows pickers
    /// that existed and were not found, which is a gap. This variant carries
    /// nothing that tells the two apart. The families of fixes
    /// weighed are on `two_lines_along_a_pipe`.
    #[error(
        "this crossing comes in two pieces and no half-space square to both surfaces can be \
         written down in f64 to tell them apart"
    )]
    CannotNameWhichPieceOfACrossingThisIs,

    /// A cone against a second cone that shares neither its axis nor its apex,
    /// a ball whose skin runs through the apex, on the axis or off it, and the
    /// shapes off the axis that a pipe or a ball is not proved to cross it in.
    ///
    /// Off the axis a pipe or a ball is **named where its shape is proved**
    /// (Q2788): two loops held apart by a flat surface through the apex, one
    /// loop round a nappe, or nothing for a ball that misses the cone, and the
    /// marcher draws them. What stays refused off the axis is each a **gap**,
    /// since the crossing is there: a pipe along one of the cone's own straight
    /// lines; a pipe steeper than the cone whose skin runs through the apex; a
    /// shallower pipe not proved to meet the cone twice along every one of its
    /// own straight lines; and a ball that touches the cone, or that every
    /// straight line of a nappe meets. `intersect.rs`'s
    /// `a_cone_and_a_pipe_off_one_axis` and `a_ball_off_a_cones_axis` say why
    /// each is refused. The first of them is not only unfinished work: a pipe
    /// along a straight line of the cone can hold that whole line, and
    /// `rcad-topo`'s `assemble.rs` answers *this crossing holds no straight
    /// line* for every cone against a pipe.
    ///
    /// The family of fixes, shortest first. (a) **A cone and a pipe on one axis
    /// is done**, and it was neither planar nor one sign question. There is no
    /// shape question in it at all: a cone is double-napped and a pipe's radius
    /// is above zero, so `slope·|h| = r` has exactly two roots and the answer is
    /// **always exactly two circles**, with no `Pos/Zero/Neg` table like the one
    /// `a_ball_centred_on_a_pipes_axis` turns on. What is asked instead is
    /// whether the pair shares an axis, and that is **two** sign questions —
    /// `‖w_c × w_p‖² = 0` for the directions and `‖(a − o_p) × w_p‖² = 0` for
    /// the apex — both of them sums of squares this crate already asks
    /// elsewhere. `where_a_cone_and_a_pipe_cross` is the whole of it. (b) **A
    /// ball centred on a cone's axis is done**: one quadratic in how far up the
    /// axis a circle sits, whose discriminant `(1 + s²)R² − s²‖c − a‖²` says
    /// two circles, one the ball touches all the way round, or none —
    /// `where_a_cone_and_a_ball_cross`. (c) **A second cone on the axis is
    /// done**: `s₁·|h| = s₂·|h − d|` is two circles, one halfway between the
    /// apexes for one slope, or the apex alone for one apex —
    /// `where_two_cones_cross`. (c′) **Two cones that share an apex with their
    /// axes leaning apart are done**, and are not refused here any more: they
    /// meet in two or four rulings through it, or at it alone, which
    /// `the_rulings_two_cones_on_one_apex_share` names as the halves the apex
    /// cuts them into. `rcad-topo`'s `assemble.rs` answers *this crossing holds
    /// a straight line* for that pair and for no other cone-and-cone pair
    /// (§50.2 item 4). (d) **A pipe or a ball off the axis is done where its
    /// shape is proved**, and it took no eighth-degree discriminant: a pipe is
    /// read along its own straight lines and a ball along the cone's, so each
    /// shape is a few signs of quadratics, and the marcher was taught a cone to
    /// draw the loops. (e) Two cones that share neither axis nor apex are the
    /// general case left, and nothing tells their loops apart yet (Q2667).
    ///
    /// **A ball through the apex is the one coaxial pair left here**, and it is
    /// not a degree-four curve: the crossing is one circle and the apex apart
    /// from it, and `SurfaceMeet` has no answer that says a curve and a point
    /// at once. A gap of the vocabulary rather than of the arithmetic. Off the
    /// axis a ball through the apex is refused too, before any shape is asked:
    /// `a_ball_off_a_cones_axis` asks where the apex stands first. `walk`
    /// also refuses under this name a closed run stated on two cones on one
    /// axis with one apex, which touch only there — a pair `intersect` answers,
    /// and a name that is wrong for it: `walk.rs`'s `where_two_coaxial_cones_cut`
    /// says why it stands.
    ///
    /// **Naming the coaxial cone and pipe moved no file in the STEP corpus**, and §50.11
    /// measured that before it was written: this variant is raised zero times
    /// across all 367 files, and every file holding a coaxial cone-and-pipe
    /// neighbour refuses for something else. Its consumer is a fuse of a frustum
    /// with a bar, not an import. The ball and the second cone were not measured
    /// against the corpus; `rcad-topo`'s `a_cone_hollowed_along_its_axis.rs` fuses
    /// them, and answers every rule where their circles fall off the faces and
    /// refuses every rule, in the arrangement, where a circle falls on one.
    #[error("a cone crossing another curved surface is a degree-four space curve")]
    ACrossingBetweenAConeAndAnotherCurvedSurface,

    /// A plane cuts a torus in a quartic that is not a pair of circles.
    ///
    /// A **gap**, and the largest one a torus leaves. Only two families of plane
    /// cut a torus in circles: one **square to the axis**, which cuts two
    /// circles round the axis, and one **through the axis**, which cuts the two
    /// meridian circles. Both are built. Every other plane cuts a genuine
    /// quartic — a Cassini oval, a figure of eight, a pair of Villarceau
    /// circles on the two bitangent planes — and the vocabulary here has no name
    /// for one: [`EdgeRun`](crate::EdgeRun) says straight, closed, or which
    /// piece, and [`crate::PointsAlongACrossing`] draws a closed curve either
    /// from a [`Conic`](crate::Surface) or from the marcher.
    ///
    /// The family of fixes, so that this is a demonstration and not a paragraph:
    /// (a) **the marcher already draws degree-four space curves** and is the
    /// short route — it wants a torus arm in `march.rs` (written: a torus
    /// carries its own enclosing gradient there already) and a seed per loop,
    /// which is the part that is missing, because where a plane's quartic falls
    /// into one loop, two, or a figure of eight is an eighth-degree
    /// discriminant in the plane's four numbers and nobody has derived it.
    /// (b) a closed form for the quartic's branches, which is Ferrari's method
    /// on interval coefficients and gives roots as boxes that a nearly-tangent
    /// plane widens to nothing. (c) refusing the naming and letting the
    /// arrangement cut the face some other way, which is not
    /// [`intersect`](crate::intersect)'s to decide (§24.3).
    /// **(a) is the shortest**, and its consumer is a torus fused with a block
    /// that is not square to the ring.
    #[error("a plane cuts this torus in a quartic that is not a pair of circles")]
    ACrossingBetweenAPlaneAndATorusThatIsAQuartic,

    /// A torus against a ball, a pipe, a cone, or another torus, except a
    /// ball, a pipe or a cone on its own axis, which `intersect` now names
    /// whether it reaches the tube, rests on it, or misses it (row Q1376),
    /// and a pipe along its axis but off it that is clear of the tube, which
    /// it names as meeting it nowhere (row Q1975).
    ///
    /// A **gap** and the same one two leaning pipes have (§29), one degree
    /// worse: the crossing is a space curve of degree up to eight, naming which
    /// of its shapes it takes has no discriminant anybody has written down, and
    /// the marcher has no seed to start from.
    ///
    /// The family of fixes, shortest first. (a) The **coaxial** cases are
    /// circles square to the axis, found by one sign question per partner and
    /// per cone nappe: up to two for a pipe or a ball and up to four for a
    /// cone. `intersect` finds them and names each circle by its plane
    /// (row Q1334, then Q1376 for the rest of the axis), and
    /// `three_surfaces` meets a plane holding the axis there the same way
    /// (§378). `walk.rs` now draws an arc of one of these circles too
    /// (Q1396, `where_a_ring_and_a_surface_on_its_axis_cut`), and refuses
    /// only a run right round a whole circle named by the two faces alone,
    /// which `rcad-topo`'s `Model::validate` does not yet ask for — that
    /// function's own doc says why. A second torus about the same axis is
    /// still not named at all. (b) A pipe **along the axis but off it** misses
    /// the tube exactly where every distance from the axis it reaches is past
    /// the tube's, short of it, or round the whole of it, three signs of one
    /// polynomial, and `intersect` answers it as nothing (row Q1975, a pin
    /// standing beside an O-ring). One that reaches the tube, or rests on it
    /// at one point, is still refused. (c) The general case wants §29's whole
    /// second stage over again with a torus in it.
    #[error("a torus crossing another curved surface is a space curve of degree up to eight")]
    ACrossingBetweenATorusAndAnotherCurvedSurface,

    /// A point cannot be worked out as close to a crossing as was asked for.
    ///
    /// Two knobs meet here and they do different jobs: the deflection says how
    /// close, and the policy says what happens when the arithmetic cannot prove
    /// it (§23.5). A certified caller is refused; a tolerant one — which has
    /// already said how far out of place it will let a point be — gets its
    /// points with the deviation reported beside them.
    #[error(
        "a point along this crossing can only be placed within {closest} of it, and {asked} was \
         asked for"
    )]
    CannotPlaceAPointThatCloseToACrossing {
        /// The deflection the caller asked for.
        asked: f64,
        /// The closest a point can actually be placed.
        closest: f64,
    },

    /// An end of a run does not say where it is.
    ///
    /// A run's ends are [`Point`](rcad_predicates::Point)s, so an end may be a
    /// box rather than a triple — and one particular box, the one whose every
    /// side runs the whole number line, is how a pair of surfaces says it names a
    /// curve and nothing on it. **Every other end has to have a middle.** A
    /// caller who hands over a box with one infinite side, or an exact triple
    /// with a coordinate that is not a number, has said nothing about where the
    /// run starts, and the angle worked out from it is not a number either.
    ///
    /// Bad caller input rather than a broken invariant (§4.1). [`walk`] is
    /// public, [`NamedRun`] has a public constructor, and nothing between them
    /// narrows what an end may be — which is §16.10's rule about checking the
    /// public API rather than today's callers.
    ///
    /// [`walk`]: crate::walk
    /// [`NamedRun`]: crate::NamedRun
    #[error("an end of this run does not say where it is")]
    AnEndOfARunNamesNoPlace,

    /// A run that goes right round a closed curve does not come back to where
    /// it started.
    ///
    /// A closed run comes back to where it started by definition, so a pair of
    /// different ends on one names a curve that is not there. Bad caller input
    /// rather than a broken invariant: [`NamedRun::new`] is public, and
    /// `rcad-topo`'s `Model` — the crate that calls it with a stored edge's
    /// ends — is `Deserialize`, so the two ends an edge's stored curve
    /// carries are whatever the bytes say, not something this constructor
    /// was ever handed a chance to check earlier (CLAUDE.md's rule on panics
    /// vs. bad caller input, not §16.11 — that section is about the
    /// dynamic-range check being a ratio and says nothing about this).
    ///
    /// [`NamedRun::new`]: crate::NamedRun::new
    ///
    /// Boxed: a bare pair of [`Point`](rcad_predicates::Point)s pushed this
    /// enum's largest variant past what `clippy::result_large_err` allows,
    /// and every other variant here is small.
    #[error(
        "a run that goes right round comes back to where it started, but {start:?} and {end:?} \
         are different points"
    )]
    AClosedRunDoesNotComeBackToWhereItStarted {
        /// Where the run says it starts.
        start: Box<rcad_predicates::Point>,
        /// Where the run says it ends.
        end: Box<rcad_predicates::Point>,
    },

    /// A piece of a crossing was named with no ends and nothing picking it
    /// out.
    ///
    /// [`EdgeRun::PartOfWhereItCrosses`](crate::EdgeRun::PartOfWhereItCrosses)
    /// exists to say *which* piece of a crossing a run is; with no ends and
    /// no half-space it says nothing at all.
    ///
    /// Since §215 it also answers a closed run whose two faces are the one cone.
    /// That pair names no crossing, so the run's own end is the only thing left
    /// to put the circle through, and a run with no ends leaves nothing.
    #[error("a piece of a crossing with no ends and nothing picking it is not a piece of anything")]
    ACrossingPieceHasNoEndsAndNothingPickingIt,

    /// How big the curve where two surfaces cross is could not be worked out,
    /// because a product it needs falls out of `f64`'s range.
    ///
    /// A **gap** and not a limit, and the same one the naming half has: the same
    /// two surfaces written higher up the number line are walked without
    /// complaint.
    #[error("how big this crossing is needs a product that falls out of f64's range")]
    CannotWorkOutHowBigACrossingIs,

    /// A crossing could not be followed because the two surfaces are too nearly
    /// parallel where it runs.
    ///
    /// The marcher's near-tangency branch, and the one roadmap §6.3 names as the
    /// place kernels produce silent garbage. Following a crossing needs a
    /// direction, and two surfaces that lean against each other at a shrinking
    /// angle give a shrinking direction and a growing bound on where the points
    /// really are. Past the point where either can be worked out at all, this.
    ///
    /// A **gap** where the crossing is a real curve and the arithmetic ran out
    /// of angle, and a **limit** where the two surfaces are genuinely tangent
    /// along it, because then there is no direction to be had at any precision.
    /// Which one is not known here, so it is neither. A crossing that touches
    /// itself at a point is named as two loops that meet there, and a run with
    /// that point for a corner is refused this today where the corner is
    /// settled onto both surfaces, though a corner where the walls touch is a
    /// real place on the crossing: there it is a gap (Q2649).
    ///
    /// **It carries the two surfaces and which question gave up on them**,
    /// because a refusal that carries neither cannot be told from the three
    /// other things the marcher used to raise it with, and a caller holding a
    /// whole solid has no way back to the pair. `docs/decisions.md` §110, and
    /// §109.1 for the 44 corpus files whose verdicts rested on a message that
    /// named the enclosing solid and nothing else.
    ///
    /// Boxed for the same reason the closed-run variant above is: a bare pair of
    /// [`Surface`](crate::Surface)s is the largest thing in this enum by a wide
    /// margin, and `clippy::result_large_err` measures the whole `Result`.
    #[error("this crossing runs where its two surfaces are too nearly parallel to follow it, {at}")]
    CannotFollowACrossingWhereItsSurfacesAreTangent {
        /// The two surfaces whose crossing was being followed, **in the frame
        /// whoever asked was working in** — the same numbers a caller of
        /// [`walk`](crate::walk) passed, and a caller of
        /// `SolidView::mesh` its own model's.
        ///
        /// A layer that works in a frame of its own moves them back with
        /// [`Error::with_its_surfaces_moved`] before handing the refusal on,
        /// which is what the mesher does and `docs/decisions.md` §116 is the
        /// argument for.
        surfaces: Box<[crate::Surface; 2]>,
        /// Which of the marcher's questions ran out of angle.
        at: WhereFollowingACrossingGaveUp,
    },

    /// A crossing could not be followed because no step along it, however short,
    /// stayed on it.
    ///
    /// **A different wall from the one above, and eight powers of ten away from
    /// it** (`docs/decisions.md` §109.6). The three questions that read the two
    /// normals — settling a point, bounding how far off it is, and crossing the
    /// normals for a direction — all answer without complaint at the tightest
    /// lean a ball on a pipe can be given, `2.1e-8`, and this is what refuses
    /// there. Sharing one name with them is what made the corpus census count
    /// stepping failures as tangency.
    ///
    /// Whether this is a gap or a limit is **not settled**. The shortening
    /// budget's own doc comment says reaching the end of it "means the curve is
    /// not being followed rather than that the budget was tight", and §109.6
    /// turned that sentence back into the open question it is: at a lean the
    /// geometry can really present, the other two questions answer fine.
    /// §16.10 wants a demonstration either way and there is not one yet.
    #[error("no step along this crossing stayed on it, after {shortenings} shortenings")]
    CannotFollowACrossingBecauseNoStepStaysOnIt {
        /// The two surfaces whose crossing was being followed, in the frame
        /// whoever asked was working in — see the variant above for what that
        /// means and who has to do something about it.
        surfaces: Box<[crate::Surface; 2]>,
        /// How many times the step was shortened before the marcher gave up.
        shortenings: usize,
    },

    /// A crossing too small, where it sits, for any `f64` point to be on both
    /// the surfaces that make it.
    ///
    /// Two pipes leaning across each other cross in a loop as wide as the
    /// thinner one. Put that loop far enough from the origin and the numbers
    /// there are spaced further apart than the loop is wide, so the nearest
    /// point that can be written down is off one of the two surfaces by a great
    /// deal more than the loop's own size. There is nothing to walk.
    ///
    /// A **limit** and not a gap: the curve is real, but no arithmetic of this
    /// width can name a point on it, and the same pair with a thicker pipe or
    /// nearer the origin walks without complaint.
    /// `how_small_a_crossing_can_be_before_no_point_sits_on_it.rs` is the
    /// demonstration rather than this paragraph, and `docs/decisions.md` §94 has
    /// the measurement.
    #[error("this crossing is smaller than the gap between the numbers where it sits")]
    ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces,

    /// A crossing was asked to be walked into more points than one is allowed to
    /// hold.
    #[error("walking this crossing that finely would take more than {most} points")]
    TooManyPointsAlongOneCrossing {
        /// The most one crossing is walked into.
        most: usize,
    },

    /// Two surfaces that cross in a curve that runs on for ever rather than in a
    /// closed one.
    ///
    /// A closed edge needs one vertex on it and
    /// [`where_a_closed_crossing_starts`](crate::where_a_closed_crossing_starts)
    /// is where it goes. Two planes, a plane down a pipe's own axis, and two
    /// pipes pointing the same way cross in lines that run on for ever, and a
    /// line has no place on it that the pair alone picks out. **A plane and a
    /// cone crossing in a parabola or a hyperbola get this too**, and it is the
    /// honest name for what is wrong with such a run: those two shapes are
    /// walked without complaint, and what they have no answer for is only the
    /// question this error is asked by. A caller that *states* a run as closed
    /// on one of them gets it from [`walk`](crate::walk) as well, because a
    /// crossing that does not close is not a piece of what that run named.
    ///
    /// A **limit** and not a gap (§16.10): the vertex this would name does not
    /// exist, because the edge those pairs contribute is cut by corners at both
    /// ends rather than closing on itself.
    #[error("these two surfaces cross in a curve that runs on for ever rather than a closed one")]
    TheCrossingOfTheseTwoSurfacesIsNotClosed,

    /// Two surfaces whose crossing is not one closed curve lying in a plane.
    ///
    /// Ordering points round a closed crossing rests on the curve being convex,
    /// which is what makes three points' cyclic order the sign of the area they
    /// sweep. Two planes cross in a straight line, a plane along a pipe's axis
    /// cuts two of them, and a ball against a pipe or a pipe against a pipe
    /// crosses in a degree-four space curve that is neither flat nor convex.
    ///
    /// Four kinds of those quartic loops are ordered all the same, and never
    /// get this: the two loops a ball off a pipe's axis crosses it in where it
    /// swallows the pipe's whole cross-section (Q1554), and the two a fat pipe
    /// crosses a thin one it swallows in (Q1483), or that a thin pipe resting
    /// on its inside crosses it in, which meet where the two touch (Q2709),
    /// each read down the pipe's axis; and the one loop a ball off a pipe's
    /// axis crosses it in where it reaches into its side (Q1767), and the one
    /// loop two pipes cross in where neither swallows the other (Q1880), as a
    /// thin pipe reaching out of a fat one's side leaves, each read round a
    /// line.
    ///
    /// A **limit** for the straight cases — the question is a different one and
    /// the arrangement's dominant-axis rule already answers it. A ball off a
    /// pipe's axis that touches the pipe from inside crosses it in a figure of
    /// eight and is refused too; which of the two that is has not been argued.
    /// A thin pipe touching a fat one from inside was refused the same way
    /// until Q2709, and was a gap: each of its two loops still runs once round
    /// the thin pipe's axis, so it is ordered as a swallowed pipe's loops are.
    #[error("these two surfaces do not cross in one closed curve lying in a plane")]
    TheseTwoSurfacesDoNotCrossInAConic,

    /// Two surfaces that are the one surface written twice, asked where they
    /// cross.
    ///
    /// Two balls with their centres in the same place and one radius share every
    /// point of their skin and cross nowhere. The radius of the circle where two
    /// balls cross is a ratio whose divisor is the gap between the two centres,
    /// so a gap of nothing divides by nothing: `0 / 0` is `NaN`, every coordinate
    /// of the circle's middle is `NaN`, and what a caller used to get back was
    /// [`Error::NotFinite`] naming "point on a plane" — a refusal about the
    /// radical plane rather than about the pair that has none.
    ///
    /// A **limit** and not a gap (§16.10): the circle this asks for does not
    /// exist at any precision. A pair of faces that really are on one ball is a
    /// seam, and a seam's run carries the second surface itself — see
    /// [`EdgeRun::PartOfWhereItCrosses`](crate::EdgeRun::PartOfWhereItCrosses)
    /// and [`EdgeRun::RightRoundWhereItCrosses`](crate::EdgeRun::RightRoundWhereItCrosses).
    #[error("these two surfaces are one surface written twice, and one surface crosses nothing")]
    TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing,

    /// Three surfaces of which fewer than two are flat, and no road back to two
    /// of them.
    ///
    /// Two planes cross in a line, and where a quadric cuts that line is a
    /// quadratic in one unknown whose discriminant is a polynomial in the stored
    /// numbers. One plane and two quadrics is a quartic in general, and three
    /// quadrics is a degree-eight system.
    ///
    /// **"In general" is doing real work in that sentence**, and
    /// `three_surfaces.rs`'s module doc names twelve families it answers. In
    /// seven, the plane and two quadrics are no quartic at all: two quadrics
    /// that square `p` the same way differ by something flat; two surfaces
    /// turned about one axis are cut
    /// by a plane square to it in two circles about one centre, which are one
    /// circle or share nothing; the same two share circles square to the axis,
    /// which a plane holding the axis cuts in two points each; a plane that
    /// misses or barely touches one of the two settles the triple with a sign;
    /// a plane beside that axis, parallel to it or leaning to it by no more
    /// than half a right angle, cuts each shared circle in a chord, touches it
    /// or misses it, as an exact sign at its height says; a plane that cuts
    /// both in circles about two centres — a ball anywhere, a pipe or a cone
    /// square to it — cuts them in two circles that differ by a line; and a
    /// plane along a pipe cuts it in lines, each of which meets the other
    /// surface where a quadratic is zero. Two pipes running the same way and a
    /// cap plane — a union of two overlapping parallel bars — is the first, a
    /// bar through a ball and a cap across both is the second, a wall through
    /// the circle a frustum crosses a bar in is the third, a ball and a bar
    /// both resting on a block's floor is the fourth, a D-flat on a shaft
    /// across the circle a chamfer cone crosses it in, even where a turn leaves
    /// the flat a rounding off parallel to the shaft (Q2177), is the fifth, a
    /// bar's cap across a ball off its axis is the sixth, and a wall through
    /// the axes of two bars that cross is the seventh. All seven are answered,
    /// and so are
    /// three curved surfaces two of which square `p` the same way, since the
    /// plane those two differ by takes one of their places (Q1284): two cones of
    /// one slope about one axis and a ball. The eighth is a ring with a ball, a
    /// pipe or a cone on its axis and a plane that holds that axis or is square
    /// to it:
    /// a bend's end cap across a bar of the bend's own radius, and a bead's
    /// flat face on a shaft of its outer radius (Q1566).
    ///
    /// The ninth and tenth are quartics, and need no algebraic solver. A plane cuts
    /// a ball, a pipe or a cone in a curve that keeps within a band about where
    /// the axis crosses it, and where the bands of the two are proved apart the
    /// three share nothing (Q2127). Where they overlap and the plane is near
    /// enough square to both axes, one curve is a line bent by little across
    /// the other, and the two cross at two points or nowhere (Q2155) — a cap a
    /// rounding off square to a bar and its chamfer cone, which
    /// [`Isometry::compose`](crate::Isometry::compose)'s rounding puts on a
    /// part placed twice. A ball whose centre is on a pipe's or a cone's axis
    /// is answered exactly instead, since the two touch along a circle on a
    /// plane known exactly, and that plane takes the other one's place: a
    /// slanted cut through where a rod's round end meets it.
    ///
    /// The eleventh is a plane a rounding off parallel to a pipe's axis and
    /// farther from it than the pipe's radius, which reaches the pipe only
    /// `10¹⁶` or more along it. A ball, a second pipe leaning to the first or a
    /// cone with no ruling along the pipe is far off there, so the three share
    /// nothing: the side of a block three from a bar and its chamfer cone, all
    /// placed by one turn (Q2156).
    ///
    /// The twelfth is a plane that one wall mirrors with both curved surfaces:
    /// the wall through the axis of a pipe or a cone and the plane's normal,
    /// where it holds the other surface's centre or axis too. On the plane the
    /// two then differ by a quadratic in the height up that axis alone, and
    /// the points are the ends of the chords at its roots (Q2196). The cut
    /// `x + 2z = 6` across a bar of radius three up `z` and the cone of slope
    /// one from the origin, steeper than half a right angle to their axis, is
    /// it, at `(0, ±3, 3)`; so is the floor across a pipe of radius twelve
    /// along `(0, 3, 4)` and a ball of thirteen about `(0, 3, 4)`, whose stored
    /// direction rounds and leaves the ball's centre a rounding off the axis,
    /// at `(±12, 0, 0)`; and so is a ball resting on a rod, cut leaning through
    /// where the two touch.
    ///
    /// This is what is left over — among it a plane leaning well off square to
    /// the axis of a pipe or a cone, which cuts it in a curve too far from a
    /// line for the road for crossing curves, against a second round surface
    /// off the wall through that axis and the plane's normal, such as the plane
    /// `x + 16z = 8` against a pipe of radius one along `x` and a ball of
    /// radius three half a unit off that axis, which meet in four points
    /// (`a_ray_along_a_balls_skin_leaning_to_a_pipe_off_its_centre_is_refused_with_four_points_there`
    /// in `rcad-topo`), or a bar's cap across a second bar leaning to it whose
    /// axis passes the first one's by; two curves that touch rather than
    /// cross, where no ball is centred on the other surface's axis and no wall
    /// mirrors all three; a plane a rounding off parallel to a pipe's axis
    /// that cuts the pipe, against a second round surface off that axis and
    /// off that wall, which the road along a pipe takes only where it is
    /// exactly parallel; and three curved surfaces two of which differ by a
    /// plane that no stored [`Plane`](crate::Plane) is exactly, such as two
    /// balls whose centres lie along a diagonal.
    ///
    /// A **gap** and not a limit (§16.10): the points are there, and a real
    /// algebraic solver reaches them. The family of fixes is the
    /// Dupont–Lazard–Lazard–Petitjean machinery or a certified numerical solver
    /// with a separation bound of its own, and both are a milestone rather than
    /// a lane (`docs/decisions.md` §31.2). The consumer that calls it due is
    /// named there too: two capped bars crossing near one of their caps. For
    /// the plane no stored plane is, the fix is smaller: roads that take the
    /// plane as the exact numbers it is, as the first family's road already
    /// does. `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved`,
    /// `a_cap_across_a_leaning_bar_meets_it_where_their_axes_cross_and_is_refused_where_they_pass`,
    /// `two_balls_whose_plane_no_stored_plane_is_and_a_bar_are_still_refused`
    /// and the `rcad-topo` pin above are the demonstrations that the points
    /// exist.
    #[error(
        "where three surfaces meet is only worked out when two of them are flat, and fewer than \
         two of these are"
    )]
    WhereThreeSurfacesMeetNeedsTwoOfThemFlat,

    /// A line that is only known as a box, asked where it meets a torus.
    ///
    /// Where a line meets a quadric is read off a discriminant, and a
    /// discriminant that is a box still says which way it points whenever the
    /// box stays clear of zero. A torus is a **quartic**, and its roots are not
    /// read off anything: they are found by evaluating the quartic's sign at
    /// places along the line and narrowing where the sign changes
    /// ([`real_roots`](crate::real_roots)). A coefficient that is a box has no
    /// sign to evaluate anywhere near a root, so the search has nothing to
    /// bisect on.
    ///
    /// **Not known** (§437.6). Where the line's own six numbers each name an
    /// `f64` — which is what a corner on the lattice comes to — the roots are
    /// found, so a box that holds one line is a gap. But a box can hold lines
    /// that meet the tube and lines that miss it, and for such a box there is
    /// no single answer to give: the ball refuses a ray whose box straddles
    /// its skin for that reason (`where_a_line_meets_a_surface.rs`,
    /// `a_ray_whose_box_straddles_the_skin_is_refused_rather_than_guessed`).
    /// This variant carries nothing that says which kind of box it was handed.
    /// The family of fixes is (a) bisecting on interval
    /// signs and handing back the stretches where the sign is undecided, which
    /// is [`MeetsAt::MaybeATouch`](crate::MeetsAt::MaybeATouch)'s shape applied
    /// to a whole bracket rather than to a turning point, and is a lane;
    /// (b) exact rational line data, which wants an arithmetic this kernel does
    /// not have; (c) narrowing the box first, which is not this question's to
    /// do — the box is what the caller knows.
    #[error("where a line meets a torus is found by exact signs, and this line is only a box")]
    WhereALineMeetsARingNeedsTheLineExactly,

    /// A predicate refused the input it was handed — three planes with no
    /// single meeting point, for instance.
    #[error(transparent)]
    Predicate(#[from] rcad_predicates::Error),

    /// The operation ran out of things it could prove.
    ///
    /// Carries the records of every decision that could not be settled, so the
    /// caller can see which predicate gave up and how wide its enclosure was.
    ///
    /// **A different tier from every variant above it**, and the two read very
    /// differently in a breakdown of what gave up (§17.7). Everything else here
    /// says the caller handed over numbers that describe no shape; this says the
    /// shape is fine and the arithmetic could not reach the answer. Only one of
    /// those is worth going and fixing.
    #[error("could not settle {} decisions the answer depends on", records.len())]
    Undecidable {
        /// The decisions that could not be settled.
        records: rcad_predicates::ProvenanceLog,
    },
}

impl Error {
    /// This refusal with whatever surfaces it carries moved by `movement`.
    ///
    /// **For a caller that worked in a frame of its own and has to hand the
    /// refusal back in the frame it was asked in.** Nothing in this crate needs
    /// it: whoever calls [`walk`](crate::walk) is handed back a refusal about
    /// the very surfaces it passed in. `rcad-topo`'s mesher is the caller that
    /// does — it shifts a face's numbers near the origin before walking its
    /// edges, because the crossing evaluator charges its rounding against the
    /// biggest number it was handed (§17.3, §39.7 item 2). So the pair the
    /// marcher gave up between is a rigid motion away from the pair the model
    /// holds, and a caller that matched
    /// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] to read
    /// `surfaces` got numbers that were not its model's.
    /// `docs/decisions.md` §113.6 recorded that and §116 is the argument for
    /// fixing it here.
    ///
    /// **The pair is moved rather than swapped for the pair that was handed
    /// over.** A caller knows which two surfaces it passed and could put those
    /// back exactly, with no rounding at all — but they are not always the pair
    /// the refusal is about. [`walk`](crate::walk) reads the second surface off
    /// the run itself where an edge has one face on both sides, which is what a
    /// seam is, so a swap would name a surface the marcher never looked at.
    /// Moving names the pair it did look at, and costs one rounding per
    /// surface.
    ///
    /// **Only the two variants that carry a `Surface` are named, and a new one
    /// is not added here by default** — the same rule, and for the same reason,
    /// as `rcad-topo`'s `walking_along_this_edge_refused`: a refusal carrying
    /// geometry has to be *decided* about rather than swept in. Everything else
    /// is handed back as it came, because everything else says nothing about
    /// where anything is.
    ///
    /// An `if let` over the two rather than a `match` with a last arm, because
    /// that last arm would be a wildcard over this crate's own enum and §40 is
    /// about where the next variant goes to be ignored. Naming the two is the
    /// whole of the rule above, and this spelling is the one that cannot grow a
    /// silent third case.
    #[must_use]
    pub fn with_its_surfaces_moved(mut self, movement: &crate::Isometry) -> Self {
        if let Self::CannotFollowACrossingWhereItsSurfacesAreTangent { surfaces, .. }
        | Self::CannotFollowACrossingBecauseNoStepStaysOnIt { surfaces, .. } = &mut self
        {
            surfaces[0] = surfaces[0].transformed(movement);
            surfaces[1] = surfaces[1].transformed(movement);
        }
        self
    }

    /// Whether the answer this refusal stands in for exists (§16.10).
    ///
    /// `docs/decisions.md` §437.6. A **limit** says no answer this kernel can
    /// hold exists, a **gap** that one exists and the arithmetic did not reach
    /// it, and **not known** that nobody has shown either. Every limit and gap
    /// claimed here is reached and asserted by
    /// `what_a_geometry_refusal_says_about_its_answer.rs`.
    ///
    /// A variant that can be either kind, and carries nothing that tells them
    /// apart, says not known: [`Error::NotFinite`] is a `NaN` the caller
    /// handed over, or a weighted circle of radius `2e307` whose control point
    /// overflowed while the circle itself is representable. Bad input with no
    /// shape behind it is a limit, and so is what today's kernel cannot hold,
    /// such as a degree past [`MAX_DEGREE`](crate::MAX_DEGREE) (§437.6).
    ///
    /// One match and no wildcard, so a new variant does not compile until it is
    /// given a class.
    #[must_use]
    pub fn limit_or_gap(&self) -> LimitOrGap {
        match self {
            Self::QuarterTurnsNeedAnAxisAlongXYOrZ { .. }
            | Self::RadiusNotPositive { .. }
            | Self::SlopeNotPositive { .. }
            | Self::HalfAngleIsNotACone { .. }
            | Self::ATorusThatRunsThroughItself { .. }
            | Self::DeflectionNotPositive { .. }
            | Self::SegmentHasNoLength
            | Self::ParameterOutsideSegment { .. }
            | Self::NominalSizeNotPositive { .. }
            | Self::FeatureSizeNotPositive { .. }
            | Self::DynamicRangeTooWide { .. }
            | Self::DegreeNotSupported { .. }
            | Self::TooFewKnots { .. }
            | Self::KnotsGoBackwards { .. }
            | Self::KnotRepeatedTooOften { .. }
            | Self::KnotVectorHasNoSpan
            | Self::WrongNumberOfControlPoints { .. }
            | Self::WeightNotPositive { .. }
            | Self::ParameterOutsideDomain { .. }
            | Self::CannotRepeatKnotAgain { .. }
            | Self::NothingOnOneSideOfTheSplit { .. }
            | Self::AnEndOfARunNamesNoPlace
            | Self::AClosedRunDoesNotComeBackToWhereItStarted { .. }
            | Self::ACrossingPieceHasNoEndsAndNothingPickingIt
            | Self::ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces
            | Self::TheCrossingOfTheseTwoSurfacesIsNotClosed
            | Self::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing => LimitOrGap::Limit,
            Self::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo
            | Self::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
            | Self::ACrossingBetweenATorusAndAnotherCurvedSurface
            | Self::CannotWorkOutHowBigACrossingIs
            | Self::WhereThreeSurfacesMeetNeedsTwoOfThemFlat => LimitOrGap::Gap,
            Self::NotFinite { .. }
            | Self::ZeroLength { .. }
            | Self::PointsAreInALine
            | Self::KnotsTooFarApart { .. }
            | Self::KnotVectorNotClamped
            | Self::WeightTooSmallToBlend { .. }
            | Self::CannotNameWhichPieceOfACrossingThisIs
            | Self::ACrossingBetweenAConeAndAnotherCurvedSurface
            | Self::CannotPlaceAPointThatCloseToACrossing { .. }
            | Self::CannotFollowACrossingWhereItsSurfacesAreTangent { .. }
            | Self::CannotFollowACrossingBecauseNoStepStaysOnIt { .. }
            | Self::TooManyPointsAlongOneCrossing { .. }
            | Self::TheseTwoSurfacesDoNotCrossInAConic
            | Self::WhereALineMeetsARingNeedsTheLineExactly
            | Self::Predicate(..)
            | Self::Undecidable { .. } => LimitOrGap::NotKnown,
        }
    }
}

/// What a refusal says about the answer it stands in for: none to be had, one
/// not reached, or not known (`docs/decisions.md` §16.10, §437.6).
///
/// Its words come from [`Display`](core::fmt::Display), so a caller outside
/// this crate reads the class without matching this enum and keeping a
/// wildcard arm for its next variant.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitOrGap {
    /// No answer this kernel can hold exists, so there is nothing to fix.
    Limit,
    /// An answer exists, and the arithmetic did not reach it.
    Gap,
    /// Nobody has shown which of the two it is, or the refusal carries nothing
    /// that tells them apart.
    NotKnown,
}

impl core::fmt::Display for LimitOrGap {
    fn fmt(&self, into: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        into.write_str(match self {
            Self::Limit => "limit",
            Self::Gap => "gap",
            Self::NotKnown => "not known",
        })
    }
}

/// Which of the marcher's questions ran out of angle between two surfaces.
///
/// Four questions rather than four lines of source: a line number goes stale on
/// the next edit, and what a reader of a refusal wants to know is *what was
/// being asked* when the angle ran out. Two of the four read the two normals
/// directly, one divides by the sine between them, and one is not the marcher's
/// at all — it is the ordering question a plane resting along a ring's tube has
/// no answer to.
///
/// **This is what tells the honest tangency apart from the near-tangency the
/// arithmetic merely could not reach**, which is the distinction §16.10 asks for
/// and the one the corpus census could not make (`docs/decisions.md` §109.2).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhereFollowingACrossingGaveUp {
    /// Settling a point onto both surfaces: the two normals lean so little that
    /// the Newton correction has nothing left to divide by.
    SettlingAPointOntoBothSurfaces,
    /// Bounding how far a point really sits off the crossing: the sine of the
    /// angle between the two surfaces could not be told from zero, so there is
    /// no bound to be had at all.
    BoundingHowFarOffTheCrossingAPointIs,
    /// The same bound, worked out and larger than any number there is.
    TheBoundOnHowFarOffIsBiggerThanAnyNumber,
    /// Crossing the two normals for the direction the curve runs in, which came
    /// back with no length to point anywhere with.
    CrossingTheTwoNormalsForADirection,
    /// Ordering points round the one circle a plane resting along a ring's tube
    /// touches it in, where the two surfaces touch rather than cross.
    OrderingTheCircleAPlaneRestsOnARingAlong,
}

impl core::fmt::Display for WhereFollowingACrossingGaveUp {
    fn fmt(&self, into: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let asking = match self {
            Self::SettlingAPointOntoBothSurfaces => "settling a point onto both of them",
            Self::BoundingHowFarOffTheCrossingAPointIs => "bounding how far off a point sits",
            Self::TheBoundOnHowFarOffIsBiggerThanAnyNumber => {
                "bounding how far off a point sits, which overflowed"
            }
            Self::CrossingTheTwoNormalsForADirection => "crossing the two normals for a direction",
            Self::OrderingTheCircleAPlaneRestsOnARingAlong => {
                "ordering the circle a plane rests on a ring along"
            }
        };
        write!(into, "asked while {asking}")
    }
}

/// The result of anything in this crate that can refuse.
pub type Result<T, E = Error> = core::result::Result<T, E>;
