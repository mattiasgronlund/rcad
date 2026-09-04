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
    /// Refusing loses no curve. A rational spline depends on its weights only
    /// through their ratios, so multiplying every weight by a power of two
    /// gives back the identical curve, exactly and with no bits lost.
    #[error(
        "the weight {weight:e} is too small to blend; scale every weight up by a power of two \
         for the same curve"
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

    /// Two surfaces tangent along their crossing rather than at a point, so
    /// what they cross in is one curve that runs through itself.
    ///
    /// A ball resting on the inside of a pipe from the far side, or two pipes of
    /// equal radius whose axes meet: the two loops the crossing would otherwise
    /// come in touch each other, and there is no half-space that picks one of
    /// them, because at the touching point they share a point.
    ///
    /// **A limit of the naming half taken with the evaluator, and a gap of the
    /// naming half on its own** (§16.10, §29.6). [`EdgeRun`](crate::EdgeRun)
    /// *can* say this: the two loops of a pinch are the two sides of one
    /// half-space, and the two ellipses are the parts lying in one plane, which
    /// is a pair of opposite half-spaces. What cannot be written down is either
    /// of those planes, which is the same gap
    /// [`Error::CannotNameWhichPieceOfACrossingThisIs`] is about.
    ///
    /// The reason to refuse here rather than there is the evaluator. Where a
    /// crossing runs through itself the two surfaces are **tangent**, so the
    /// crossing has no direction and the marcher has nothing to follow — and
    /// naming two pieces nothing can draw is the half-answer §24.10 refuses
    /// whole. `marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow`
    /// is the demonstration.
    ///
    /// The family of fixes: a marcher that can be told to start away from a
    /// singular point and stop at it, and a third `EdgeRun` variant that hands a
    /// fuse the crossing point rather than making it notice. Both have one
    /// consumer and it is M4's fuse.
    #[error(
        "these two surfaces are tangent along where they cross, so what they cross in is one \
         curve that runs through itself"
    )]
    ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo,

    /// A crossing came in two pieces and no half-space could be written down
    /// that picks one of them out and holds along the whole of it.
    ///
    /// A **gap** and not a limit (§16.10): the two pieces are there and they are
    /// tellable apart, and what runs out is `f64`'s supply of unit vectors
    /// exactly square to a given direction. The families of fixes weighed are on
    /// `two_lines_along_a_pipe`.
    #[error(
        "this crossing comes in two pieces and no half-space square to both surfaces can be \
         written down in f64 to tell them apart"
    )]
    CannotNameWhichPieceOfACrossingThisIs,

    /// A cone against a ball or another cone, and a cone against a pipe that
    /// does not share its axis.
    ///
    /// A **gap** and the same one two leaning pipes have (§29): the crossing is
    /// a degree-four space curve, naming which of its shapes it takes is an
    /// eighth-degree discriminant, and drawing it needs the marcher. None of
    /// that is built for a cone.
    ///
    /// The family of fixes, shortest first. (a) **A cone and a pipe on one axis
    /// is done**, and it was neither planar nor one sign question. There is no
    /// shape question in it at all: a cone is double-napped and a pipe's radius
    /// is above zero, so `slope·|h| = ρ` has exactly two roots and the answer is
    /// **always exactly two circles**, with no `Pos/Zero/Neg` table like the one
    /// `a_ball_centred_on_a_pipes_axis` turns on. What is asked instead is
    /// whether the pair shares an axis, and that is **two** sign questions —
    /// `‖w_c × w_p‖² = 0` for the directions and `‖(a − o_p) × w_p‖² = 0` for
    /// the apex — both of them sums of squares this crate already asks
    /// elsewhere. `where_a_cone_and_a_pipe_cross` is the whole of it. (b) A ball
    /// centred on a cone's axis is the same shape of answer and is **not**
    /// built: §13 wants a named consumer and it has none. (c) A second cone
    /// sharing the axis is held out for a stronger reason than that.
    /// `rcad-topo`'s `assemble.rs` answers *this crossing holds no straight
    /// line* for the whole cone-and-cone row on the strength of this refusal,
    /// and two cones that share an apex meet along rulings, which are lines
    /// (§50.2 item 4). That arm is the first thing to change the day this one
    /// lifts. (d) The general case wants the marcher taught a cone and an
    /// eighth-degree discriminant derived, which is §29's whole second stage
    /// over again.
    ///
    /// **Naming the coaxial pair moved no file in the STEP corpus**, and §50.11
    /// measured that before it was written: this variant is raised zero times
    /// across all 367 files, and every file holding a coaxial cone-and-pipe
    /// neighbour refuses for something else. Its consumer is a fuse of a frustum
    /// with a bar, not an import.
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

    /// A torus against a ball, a pipe, a cone, or another torus.
    ///
    /// A **gap** and the same one two leaning pipes have (§29), one degree
    /// worse: the crossing is a space curve of degree up to eight, naming which
    /// of its shapes it takes has no discriminant anybody has written down, and
    /// the marcher has no seed to start from.
    ///
    /// The family of fixes, shortest first. (a) The **coaxial** cases are two
    /// circles each and are one sign question apiece — a pipe sharing the
    /// torus's axis crosses it in two circles square to that axis, and so does a
    /// ball centred on it and a second torus about it. That covers a rod through
    /// the middle of a ring, which is what a real file holds. (b) The general
    /// case wants §29's whole second stage over again with a torus in it.
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
    /// The two are told apart by the naming half:
    /// [`Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`] is what a
    /// tangency along a whole curve comes back as before anything is walked.
    #[error("this crossing runs where its two surfaces are too nearly parallel to follow it")]
    CannotFollowACrossingWhereItsSurfacesAreTangent,

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
    /// A **limit** for the straight cases — the question is a different one and
    /// the arrangement's dominant-axis rule already answers it — and a **gap**
    /// for the two quartic ones, where the curve is there and ordering points
    /// along it takes the marcher rather than a determinant (§16.10, §31.2
    /// row 8).
    #[error("these two surfaces do not cross in one closed curve lying in a plane")]
    TheseTwoSurfacesDoNotCrossInAConic,

    /// Three surfaces of which fewer than two are flat, and no road back to two
    /// of them.
    ///
    /// Two planes cross in a line, and where a quadric cuts that line is a
    /// quadratic in one unknown whose discriminant is a polynomial in the stored
    /// numbers. One plane and two quadrics is a quartic in general, and three
    /// quadrics is a degree-eight system.
    ///
    /// **"In general" is doing real work in that sentence**, and
    /// `three_surfaces.rs`'s module doc names the two
    /// families it excludes: two quadrics that square `p` the same way differ by
    /// something flat, and a plane that misses or barely touches one of the two
    /// settles the triple with a sign. Two pipes running the same way and a cap
    /// plane — a union of two overlapping parallel bars — is the first, and a
    /// ball and a bar both resting on a block's floor is the second. Both are
    /// answered, and this is what is left over.
    ///
    /// A **gap** and not a limit (§16.10): the points are there, and a real
    /// algebraic solver reaches them. The family of fixes is the
    /// Dupont–Lazard–Lazard–Petitjean machinery or a certified numerical solver
    /// with a separation bound of its own, and both are a milestone rather than
    /// a lane (`docs/decisions.md` §31.2). The consumer that calls it due is
    /// named there too: two capped bars crossing near one of their caps.
    /// `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved`
    /// and `two_bars_leaning_across_each_other_and_a_plane_are_still_refused`
    /// are the demonstrations that the points exist.
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
    /// A **gap** and not a limit: the roots exist, and where the line's own six
    /// numbers each name an `f64` — which is what a corner on the lattice comes
    /// to — they are found. The family of fixes is (a) bisecting on interval
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

/// The result of anything in this crate that can refuse.
pub type Result<T, E = Error> = core::result::Result<T, E>;
