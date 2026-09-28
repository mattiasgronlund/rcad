//! A [`Model`]'s solids become an ISO 10303-21 file, or the writer refuses by
//! the name of what it could not say.
//!
//! This is the inverse of [`Import`](crate::Import), entity for entity:
//! `manifold_solid_brep`, `closed_shell`, `advanced_face`, `face_outer_bound`,
//! `edge_loop`, `oriented_edge`, `edge_curve`, `vertex_point`. What it exists
//! for is an **oracle the reader does not supply itself** — a solid the kernel
//! built, written out and read back, has to be the same solid, and the reader
//! alone could never say so.
//!
//! # What a round trip cannot check, said before anything else
//!
//! It checks this writer against this reader. A misunderstanding of AP203 the
//! two share cancels out and passes green. That is why `step-io` is a
//! dev-dependency of this crate and reads the bytes this writer produces in
//! `crates/rcad-step/tests/a_second_reader_reads_what_this_writer_wrote.rs`, and
//! why the hand-written files in `tests/written_by_hand/` — which nothing here
//! produced — stay the check on what the *entities* mean.
//!
//! # An edge is named by the two surfaces either side of it
//!
//! `docs/decisions.md` §24.3, and it is also what a STEP `EDGE_CURVE` is. This
//! writer takes each edge's two neighbouring faces off the shell as it walks it
//! and works the curve out from their two surfaces — two planes cross in a
//! line, a plane square across a pipe crosses it in a circle. **Where those two
//! surfaces do not name a curve, it refuses by name and writes nothing**, and
//! [`CannotWrite::ACurveTheFacesEitherSideOfItDoNotName`] says what closes it.

use std::collections::HashMap;

use rcad_geom::{Plane, Point, Scale, Surface, Unit, Vector};
use rcad_predicates::expansion::two_product;
use rcad_topo::{EdgeKey, Model, ShellView, VertexKey, WireStepView};

use crate::geometry::plane_through;

/// Write every solid in `model` as an ISO 10303-21 file called `called`.
///
/// The unit is [`Scale::unit`] and no coordinate is converted on the way out —
/// §3.5 again, from the other side: a model in inches writes a file in inches.
///
/// # Errors
///
/// A [`CannotWrite`] naming what this writer could not say. Every variant is
/// classified a limit or a gap on itself (§16.10).
pub fn write(model: &Model, called: &str) -> Result<String, CannotWrite> {
    if model.solid_count() == 0 {
        return Err(CannotWrite::TheModelHasNoSolid);
    }
    let mut writing = Writing::new(called, model.scale())?;
    for solid in model.solids() {
        let mut shells = solid.shells();
        let shell = shells.next().ok_or(CannotWrite::ASolidWithNoShell)?;
        let extra = shells.count();
        if extra > 0 {
            return Err(CannotWrite::ASolidWithMoreThanOneShell { shells: extra + 1 });
        }
        writing.solid_from_shell(model, shell)?;
    }
    Ok(writing.finished())
}

/// What stopped a [`Model`] from being written as a Part 21 file.
///
/// A **limit** is where the file has nothing to say; a **gap** is where it does
/// and this writer could not reach it (§16.10). Every gap names what closes it.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CannotWrite {
    /// The model holds no solid.
    ///
    /// **Limit.** A Part 21 file with no `MANIFOLD_SOLID_BREP` in it is one
    /// [`Import::read`](crate::Import::read) refuses as
    /// [`Refusal::TheFileHoldsNoSolid`](crate::Refusal::TheFileHoldsNoSolid),
    /// so writing one would be writing a file this kernel will not read back.
    #[error("this model holds no solid, so there is no b-rep to write")]
    TheModelHasNoSolid,

    /// A solid with no shell at all.
    ///
    /// **Limit.** There is no skin to write.
    #[error("a solid in this model has no shell")]
    ASolidWithNoShell,

    /// A solid made of more than one shell — a void inside it, or two lumps.
    ///
    /// **Gap, and the same one from the third side.** `SolidData::shells` is a
    /// list, `SolidBuilder::add_shell` refuses a second by name and
    /// [`Refusal::ASolidWithAVoidInIt`](crate::Refusal::ASolidWithAVoidInIt)
    /// refuses to read one. Writing a `BREP_WITH_VOIDS` this kernel could not
    /// read back would be the only one of the three to pretend.
    #[error("a solid in this model has {shells} shells, and this writer writes one to a solid")]
    ASolidWithMoreThanOneShell {
        /// How many shells the solid holds.
        shells: usize,
    },

    /// A wire step that stands still on a corner — a ball's pole.
    ///
    /// **Gap.** An `EDGE_LOOP` is a list of `ORIENTED_EDGE`s and a pole is not
    /// one; STEP says it with a `VERTEX_LOOP`, which
    /// [`Refusal::ALoopThisReaderDoesNotWalk`](crate::Refusal::ALoopThisReaderDoesNotWalk)
    /// refuses to read, because a wire of only a pole is one step that is not an
    /// edge and neither `SolidBuilder::add_wire` nor `Invalid::WireTooShort`
    /// takes it: both ask for three steps, or one edge that closes already, or
    /// two different edges that are not both straight. Writing a file this
    /// kernel will not read back is not a round trip. What closes it is a fourth
    /// arm on that same rule, in `rcad-topo`; the two-step arm landed there with
    /// §44.11 item 3 and this one has its own consumer waiting — four NIST files
    /// stop first at a `VERTEX_LOOP`.
    #[error("a wire in this model stands still at a pole, and an EDGE_LOOP has no step for that")]
    AWireThatStandsStillAtAPole,

    /// The two faces either side of an edge do not name the curve it runs
    /// along.
    ///
    /// **Gap, and it is `EdgeView::run` being `pub(crate)`.** The model knows
    /// exactly what this edge runs along; a run is un-located geometry and
    /// handing one out wants an `EdgeRun::transformed`, which wants a
    /// `HalfSpace::transformed` in `rcad-geom` — the *same single missing
    /// function* §42.5's lane reported for a placement argument and §42.10's
    /// reported for a turned arc. This lane is its third consumer.
    ///
    /// Until then this writer derives the curve from the two surfaces either
    /// side, which is what §24.3 says an edge *is*, and that reaches every
    /// curve except the ones a surface has to be cut open along: a sphere's
    /// meridian seam runs between the two poles, which are in a line with the
    /// centre, so no plane through them is picked out by the ends.
    #[error(
        "an edge between a {first} and a {second} runs along a curve those two do not name \
         (closed: {closed})"
    )]
    ACurveTheFacesEitherSideOfItDoNotName {
        /// What the first of the two faces sits on.
        first: &'static str,
        /// What the second sits on.
        second: &'static str,
        /// Whether the edge comes back to the corner it left.
        closed: bool,
    },

    /// An edge that no face of the shell walks, or that one walks more than
    /// twice.
    ///
    /// **Limit.** A closed shell walks every edge exactly twice, and an edge
    /// walked any other number of times is not one this writer can name from
    /// the surfaces either side of it because there are not two of them.
    #[error("an edge of this shell is walked {walks} times, and a closed shell walks each twice")]
    AnEdgeThatIsNotWalkedTwice {
        /// How many coedges of the shell walk it.
        walks: usize,
    },

    /// An [`EdgeRun`](rcad_geom::EdgeRun) kind that has arrived since this
    /// writer was written.
    ///
    /// **Gap, and the twin of the one below.** Which curve an edge is written as
    /// turns on whether it comes back to the corner it left — a `LINE` between
    /// two planes, a `CIRCLE` where a plane crosses a pipe — and
    /// [`EdgeView::is_closed`](rcad_topo::EdgeView::is_closed) is what says so.
    /// A run kind this writer has never seen has no answer there, and taking it
    /// for an open edge writes a straight `LINE` where the model holds a curve:
    /// a file describing a shape that is not the one in the model, which is the
    /// same thing guessing the nearest surface would do.
    #[error("an edge in this model runs along a kind of curve this writer has no STEP entity for")]
    ARunKindThisWriterHasNoCurveFor,

    /// A [`Surface`] kind that has arrived since this writer was written.
    ///
    /// **Gap.** `Surface` grows a variant every milestone; a new one has no
    /// Part 21 entity here until somebody writes it, and guessing the nearest
    /// one would write a file describing a shape that is not the one in the
    /// model.
    #[error("a face in this model sits on a surface kind this writer has no STEP entity for")]
    ASurfaceThisWriterHasNoEntityFor,

    /// A plane whose offset no point this writer builds reads back as, bit for
    /// bit.
    ///
    /// **Gap, reached only just above the subnormals.** Where the offset is
    /// below `2⁻⁹⁶⁹` and not subnormal, the reader rounds each product of the
    /// dot and drops what it lost (`product_is_exact`), and
    /// `divided_along_the_widest` still counts that loss, so on some normals
    /// the point misses (`some_planes_just_above_the_subnormals_are_still_refused_by_name`).
    /// What closes it is counting the loss only where the reader keeps it.
    /// Planes too far out to divide by the widest component are no longer
    /// refused (`a_plane_too_far_out_to_divide_is_written_at_a_point_it_reads_back_from`).
    /// Writing the foot of the perpendicular instead would write a file that
    /// reads back as another plane.
    #[error(
        "a face in this model sits on a plane at {offset}, and this writer found no point that carries that offset exactly"
    )]
    APlaneWhoseOffsetNoWrittenPointCarries {
        /// The plane's offset from the origin.
        offset: f64,
    },

    /// A [`Unit`] that has arrived since this writer was written.
    ///
    /// **Gap.** Writing the coordinates under a unit that is not the model's
    /// would be the silent rescale §42.12 criterion 6 forbids, from the writing
    /// side; writing them under no unit at all makes a file
    /// [`Refusal::TheFileDeclaresNoLengthUnit`](crate::Refusal::TheFileDeclaresNoLengthUnit)
    /// refuses. What closes it is a Part 21 spelling for the new unit.
    #[error("this model's coordinates are in a unit this writer has no Part 21 spelling for")]
    ALengthUnitThisWriterHasNoSpellingFor,
}

/// One file being written: its text so far and the next `#id` to hand out.
struct Writing {
    text: String,
    next: u64,
}

impl Writing {
    fn new(called: &str, scale: Scale) -> Result<Self, CannotWrite> {
        let mut writing = Self {
            text: String::new(),
            next: 1,
        };
        writing.text += &format!(
            "ISO-10303-21;\n\
             HEADER;\n\
             FILE_DESCRIPTION(({}),'2;1');\n\
             FILE_NAME({},'',(''),(''),'','','');\n\
             FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\n\
             ENDSEC;\n\
             DATA;\n",
            text(&format!(
                "a {} model written by rcad",
                scale.unit().symbol()
            )),
            text(called),
        );
        writing.units(scale.unit())?;
        Ok(writing)
    }

    /// The three units, the declared uncertainty and the geometric context,
    /// handing back the `#id` of the length unit.
    ///
    /// A unit an `SI_UNIT` prefix names is written as that prefix; an inch and
    /// a foot are written the way a real file writes them, as a
    /// `CONVERSION_BASED_UNIT` over an exact number of millimetres.
    /// [`what_the_file_declared`](crate::units::what_the_file_declared) matches
    /// that number rather than multiplying by it, so nothing rounds either way.
    fn units(&mut self, unit: Unit) -> Result<(), CannotWrite> {
        let length = match unit {
            Unit::Micrometre => self.entity("(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MICRO.,.METRE.))"),
            Unit::Millimetre => self.entity("(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.))"),
            Unit::Centimetre => self.entity("(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.CENTI.,.METRE.))"),
            Unit::Metre => self.entity("(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT($,.METRE.))"),
            Unit::Inch => self.converted_from_millimetres("INCH", 25.4),
            Unit::Foot => self.converted_from_millimetres("FOOT", 304.8),
            // `Unit` grows a variant when this kernel learns a length, and a
            // model's coordinates written under the wrong unit are the silent
            // rescale §42.12 criterion 6 forbids.
            _ => return Err(CannotWrite::ALengthUnitThisWriterHasNoSpellingFor),
        };
        let angle = self.entity("(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.))");
        let solid_angle = self.entity("(NAMED_UNIT(*)SOLID_ANGLE_UNIT()SI_UNIT($,.STERADIAN.))");
        let uncertainty = self.entity(&format!(
            "UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-7),#{length},'','')"
        ));
        self.entity(&format!(
            "(GEOMETRIC_REPRESENTATION_CONTEXT(3)\
             GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{uncertainty}))\
             GLOBAL_UNIT_ASSIGNED_CONTEXT((#{length},#{angle},#{solid_angle}))\
             REPRESENTATION_CONTEXT('',''))"
        ));
        Ok(())
    }

    fn converted_from_millimetres(&mut self, called: &str, millimetres: f64) -> u64 {
        let millimetre = self.entity("(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.))");
        let measure = self.entity(&format!(
            "LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE({}),#{millimetre})",
            real(millimetres)
        ));
        self.entity(&format!(
            "(CONVERSION_BASED_UNIT({},#{measure})LENGTH_UNIT()NAMED_UNIT(*))",
            text(called)
        ))
    }

    /// One shell, as one `CLOSED_SHELL` under one `MANIFOLD_SOLID_BREP`.
    ///
    /// Two passes, and the first is why: an `EDGE_CURVE` needs the curve, the
    /// curve comes from the two faces either side of the edge, and neither face
    /// knows about the other. So the first pass collects each edge's two
    /// surfaces off the shell, and the second writes.
    ///
    /// The shell is taken as its solid handed it out and never looked up again
    /// by key: `Model::shell` sees it un-located, so a placed solid would be
    /// written where it was built rather than where it was put (§191).
    fn solid_from_shell(&mut self, model: &Model, shell: ShellView<'_>) -> Result<(), CannotWrite> {
        let mut either_side: HashMap<EdgeKey, Vec<Surface>> = HashMap::new();
        for face in shell.faces() {
            let surface = face.surface();
            for wire in face.wires() {
                for step in wire.steps() {
                    match step {
                        WireStepView::Along(coedge) => either_side
                            .entry(coedge.edge().key())
                            .or_default()
                            .push(surface),
                        WireStepView::Pole(_) => {
                            return Err(CannotWrite::AWireThatStandsStillAtAPole);
                        }
                        // `WireStepView` grows a variant every milestone, and a
                        // step written as whichever of the two it resembles is
                        // a file describing a shape that is not the one here.
                        _ => return Err(CannotWrite::AWireThatStandsStillAtAPole),
                    }
                }
            }
        }

        let mut corners: HashMap<VertexKey, u64> = HashMap::new();
        let mut edges: HashMap<EdgeKey, u64> = HashMap::new();
        let mut skin = Vec::with_capacity(shell.face_count());
        for face in shell.faces() {
            let outward = match (face.surface(), face.orientation().is_reversed()) {
                (Surface::Plane(flat), true) => Surface::Plane(flat.reversed()),
                (surface, _) => surface,
            };
            let turned = face.orientation().is_reversed() && outward.as_plane().is_none();
            let sits_on = self.surface(&outward)?;

            let mut bounds = Vec::with_capacity(face.wire_count());
            for (which, wire) in face.wires().enumerate() {
                let mut steps = Vec::with_capacity(wire.step_count());
                for step in wire.steps() {
                    let WireStepView::Along(coedge) = step else {
                        return Err(CannotWrite::AWireThatStandsStillAtAPole);
                    };
                    let key = coedge.edge().key();
                    let forwards = !coedge.orientation_on_its_edge().is_reversed();
                    let edge = match edges.get(&key) {
                        Some(&already) => already,
                        None => {
                            let (start, end) = if forwards {
                                (coedge.start(), coedge.end())
                            } else {
                                (coedge.end(), coedge.start())
                            };
                            let written = self.edge_curve(
                                model,
                                &mut corners,
                                key,
                                (start.key(), start.point()),
                                (end.key(), end.point()),
                                either_side.get(&key).map_or(&[][..], Vec::as_slice),
                            )?;
                            edges.insert(key, written);
                            written
                        }
                    };
                    steps.push(
                        self.entity(&format!("ORIENTED_EDGE('',*,*,#{edge},{})", flag(forwards))),
                    );
                }
                let ring = self.entity(&format!("EDGE_LOOP('',({}))", references(&steps)));
                let keyword = if which == 0 {
                    "FACE_OUTER_BOUND"
                } else {
                    "FACE_BOUND"
                };
                bounds.push(self.entity(&format!("{keyword}('',#{ring},.T.)")));
            }
            skin.push(self.entity(&format!(
                "ADVANCED_FACE('',({}),#{sits_on},{})",
                references(&bounds),
                flag(!turned)
            )));
        }
        let closed = self.entity(&format!("CLOSED_SHELL('',({}))", references(&skin)));
        self.entity(&format!("MANIFOLD_SOLID_BREP('',#{closed})"));
        Ok(())
    }

    /// One `EDGE_CURVE`, with the corners and the curve it needs.
    fn edge_curve(
        &mut self,
        model: &Model,
        corners: &mut HashMap<VertexKey, u64>,
        edge: EdgeKey,
        start: (VertexKey, Point),
        end: (VertexKey, Point),
        either_side: &[Surface],
    ) -> Result<u64, CannotWrite> {
        let closed = model
            .edge(edge)
            .expect("every key here came off the model being written")
            .is_closed()
            .map_err(|_| CannotWrite::ARunKindThisWriterHasNoCurveFor)?;
        let [first, second] = match either_side {
            [first, second] => [first, second],
            _ => {
                return Err(CannotWrite::AnEdgeThatIsNotWalkedTwice {
                    walks: either_side.len(),
                });
            }
        };
        let curve = self.curve(first, second, start.1, end.1, closed)?;
        let from = self.vertex_point(corners, start);
        let to = self.vertex_point(corners, end);
        Ok(self.entity(&format!("EDGE_CURVE('',#{from},#{to},#{curve},.T.)")))
    }

    /// The curve an edge runs along, worked out from the two surfaces either
    /// side of it.
    ///
    /// Three shapes, and each is the *only* curve of this kernel's vocabulary
    /// that the pair admits, which is what makes this a derivation rather than
    /// a guess:
    ///
    /// - **two planes** cross in one straight line;
    /// - **a plane square across a pipe** — its normal along the pipe's axis —
    ///   crosses it in one circle, centred where the axis meets the plane;
    /// - **a seam of one pipe**, where the pipe is on both sides, joining two
    ///   corners that stand at the same place around the axis. A plane section
    ///   of a cylinder that is not parallel to the axis meets each straight
    ///   line of the surface once, so it cannot hold two points at one angle;
    ///   a section parallel to the axis is a pair of those straight lines. So
    ///   of everything this kernel can put on a cylinder, only the straight one
    ///   joins two such corners.
    ///
    /// Everything else refuses by name, and
    /// [`CannotWrite::ACurveTheFacesEitherSideOfItDoNotName`] says why the
    /// alternative — asking the model what the edge runs along — is one
    /// function away in another crate.
    fn curve(
        &mut self,
        first: &Surface,
        second: &Surface,
        start: Point,
        end: Point,
        closed: bool,
    ) -> Result<u64, CannotWrite> {
        let refuse = || CannotWrite::ACurveTheFacesEitherSideOfItDoNotName {
            first: kind_of(first),
            second: kind_of(second),
            closed,
        };
        match (first, second) {
            (Surface::Plane(_), Surface::Plane(_)) if !closed => Ok(self.line(start, end - start)),
            (Surface::Plane(flat), Surface::Cylinder(pipe))
            | (Surface::Cylinder(pipe), Surface::Plane(flat))
                if closed =>
            {
                let along = pipe.axis().direction();
                let facing = flat.normal().dot(along);
                if flat.normal().cross(along) != Vector::zero() || facing == 0.0 {
                    return Err(refuse());
                }
                let from = pipe.axis().origin();
                let reached = (flat.offset() - flat.normal().dot(from - Point::origin())) / facing;
                Ok(self.circle(from + along * reached, along, pipe.radius()))
            }
            (Surface::Cylinder(pipe), Surface::Cylinder(other)) if !closed && pipe == other => {
                let along = pipe.axis().direction();
                let across = |corner: Point| {
                    let out = corner - pipe.axis().origin();
                    out - along * out.dot(along)
                };
                if across(start) != across(end) {
                    return Err(refuse());
                }
                Ok(self.line(start, end - start))
            }
            (Surface::Plane(_), Surface::Plane(_))
            | (Surface::Plane(_), Surface::Cylinder(_))
            | (Surface::Cylinder(_), Surface::Plane(_))
            | (Surface::Cylinder(_), Surface::Cylinder(_))
            | (Surface::Plane(_), Surface::Sphere(_))
            | (Surface::Sphere(_), Surface::Plane(_))
            | (Surface::Plane(_), Surface::Cone(_))
            | (Surface::Cone(_), Surface::Plane(_))
            | (Surface::Plane(_), Surface::Torus(_))
            | (Surface::Torus(_), Surface::Plane(_))
            | (Surface::Cylinder(_), Surface::Sphere(_))
            | (Surface::Sphere(_), Surface::Cylinder(_))
            | (Surface::Cylinder(_), Surface::Cone(_))
            | (Surface::Cone(_), Surface::Cylinder(_))
            | (Surface::Cylinder(_), Surface::Torus(_))
            | (Surface::Torus(_), Surface::Cylinder(_))
            | (Surface::Sphere(_), Surface::Sphere(_))
            | (Surface::Sphere(_), Surface::Cone(_))
            | (Surface::Cone(_), Surface::Sphere(_))
            | (Surface::Sphere(_), Surface::Torus(_))
            | (Surface::Torus(_), Surface::Sphere(_))
            | (Surface::Cone(_), Surface::Cone(_))
            | (Surface::Cone(_), Surface::Torus(_))
            | (Surface::Torus(_), Surface::Cone(_))
            | (Surface::Torus(_), Surface::Torus(_)) => Err(refuse()),
            // `Surface` grows every milestone (docs/decisions.md §40); this
            // arm is only ever reached by the variant that has not landed
            // yet, since every one that exists today has its own line above.
            _ => Err(refuse()),
        }
    }

    fn line(&mut self, from: Point, along: Vector) -> u64 {
        let at = self.cartesian_point(from);
        let way = self.direction(along);
        let vector = self.entity(&format!("VECTOR('',#{way},{})", real(1.0)));
        self.entity(&format!("LINE('',#{at},#{vector})"))
    }

    fn circle(&mut self, centre: Point, axis: Vector, radius: f64) -> u64 {
        let place = self.placement(centre, axis);
        self.entity(&format!("CIRCLE('',#{place},{})", real(radius)))
    }

    /// One of the five surfaces this kernel holds, in the parametrisation STEP
    /// asks for.
    ///
    /// Four of the five are written in the numbers the kernel already stores
    /// and read back as those numbers. **The cone is not**, and that is worth
    /// stating: STEP writes a half-angle and [`Cone`](rcad_geom::Cone) stores
    /// its tangent, so a cone goes out through `atan` and comes back through
    /// `tan`, neither of which Rust bounds. The apex is exact — a placement at
    /// the apex has radius zero and `cone_from` reads that back as the origin
    /// the file wrote, with no division — and the slope is the round trip of
    /// two library functions. `a_cone_is_the_one_surface_whose_round_trip_goes_through_a_library_function`
    /// measures what that costs.
    ///
    /// # A plane is written at a point its offset reads back from exactly
    ///
    /// STEP has no way to write a plane except as a point and a normal, and
    /// the reader dots the two back together to get the offset
    /// ([`plane_through`]), so the point decides the offset's last bit. The
    /// foot of the perpendicular, which this writer used to send, rounds each
    /// of its three coordinates and missed on 7 777 of 20 000 planes that face
    /// no axis (`the_foot_of_the_perpendicular_misses_on_more_than_a_third_of_these_planes`).
    /// [`a_point_this_reader_reads_back_as`] picks one that does not miss
    /// (`docs/decisions.md` §313).
    ///
    /// **The normal is written as the model stores it, and does not always
    /// come back.** The reader scales a `DIRECTION` to length one, and a normal
    /// a turn left an ulp or two off length one comes back moved. No point and
    /// no direction this writer could choose fixes that:
    /// `no_direction_reads_back_as_a_normal_a_turn_left_short_of_length_one`
    /// shows one that no direction reads back as. The file carries the bits;
    /// the reader has no way to keep them, and that is `rcad-geom`'s to give
    /// (§313.3).
    fn surface(&mut self, surface: &Surface) -> Result<u64, CannotWrite> {
        Ok(match surface {
            Surface::Plane(flat) => {
                let place = self.placement(a_point_this_reader_reads_back_as(flat)?, flat.normal());
                self.entity(&format!("PLANE('',#{place})"))
            }
            Surface::Cylinder(pipe) => {
                let place = self.placement(pipe.axis().origin(), pipe.axis().direction());
                self.entity(&format!(
                    "CYLINDRICAL_SURFACE('',#{place},{})",
                    real(pipe.radius())
                ))
            }
            Surface::Sphere(ball) => {
                let place = self.placement(ball.centre(), Vector::new(0.0, 0.0, 1.0));
                self.entity(&format!(
                    "SPHERICAL_SURFACE('',#{place},{})",
                    real(ball.radius())
                ))
            }
            Surface::Cone(cone) => {
                let place = self.placement(cone.apex(), cone.axis().direction());
                self.entity(&format!(
                    "CONICAL_SURFACE('',#{place},{},{})",
                    real(0.0),
                    real(cone.slope().atan())
                ))
            }
            Surface::Torus(ring) => {
                let place = self.placement(ring.centre(), ring.axis().direction());
                self.entity(&format!(
                    "TOROIDAL_SURFACE('',#{place},{},{})",
                    real(ring.ring_radius()),
                    real(ring.tube_radius())
                ))
            }
            _ => return Err(CannotWrite::ASurfaceThisWriterHasNoEntityFor),
        })
    }

    /// An `AXIS2_PLACEMENT_3D` with no reference direction.
    ///
    /// The third attribute is optional in the schema and nothing this kernel
    /// reads back needs one — an arc is picked by its two ends and its axis,
    /// never by where the parametrisation starts — so it is written `$` rather
    /// than filled in with a direction nothing chose.
    fn placement(&mut self, origin: Point, axis: Vector) -> u64 {
        let at = self.cartesian_point(origin);
        let way = self.direction(axis);
        self.entity(&format!("AXIS2_PLACEMENT_3D('',#{at},#{way},$)"))
    }

    fn vertex_point(
        &mut self,
        corners: &mut HashMap<VertexKey, u64>,
        at: (VertexKey, Point),
    ) -> u64 {
        if let Some(&already) = corners.get(&at.0) {
            return already;
        }
        let point = self.cartesian_point(at.1);
        let written = self.entity(&format!("VERTEX_POINT('',#{point})"));
        corners.insert(at.0, written);
        written
    }

    fn cartesian_point(&mut self, at: Point) -> u64 {
        self.entity(&format!("CARTESIAN_POINT('',{})", triple(at.coordinates())))
    }

    fn direction(&mut self, way: Vector) -> u64 {
        self.entity(&format!("DIRECTION('',{})", triple(way.components())))
    }

    fn entity(&mut self, body: &str) -> u64 {
        let id = self.next;
        self.next += 1;
        self.text += &format!("#{id}={body};\n");
        id
    }

    fn finished(mut self) -> String {
        self.text += "ENDSEC;\nEND-ISO-10303-21;\n";
        self.text
    }
}

/// A point on `flat` from which this crate's reader works out `flat`'s own
/// offset, bit for bit.
///
/// The reader scales the written normal to length one and dots the point with
/// it, rounding once ([`plane_through`]). So the point is built against the
/// normal the reader will use, and has at most two coordinates that are not
/// zero:
///
/// - along the normal's widest component `nᵢ`, `offset / nᵢ`, which lands the
///   point within about an ulp of the offset;
/// - along its next widest `nⱼ`, what that left over, `r / nⱼ`, where
///   `r = offset − nᵢ·(offset / nᵢ)` is worked out exactly with one fused
///   multiply-add.
///
/// The second coordinate's own rounding moves the dot product by about `2⁻⁵³`
/// of `r`, and `r` is about an ulp of the offset, so the sum lands far inside
/// the offset's rounding interval. `nⱼ` is never so small that `r / nⱼ`
/// grows: a unit normal whose widest component is not exactly ±1 has
/// `|nⱼ| ≥ 2⁻²⁷`, and one whose widest component is ±1 leaves nothing over.
///
/// **Where `offset / nᵢ` overflows**, the point is `±offset` along the widest
/// and `offset·(1 − |nᵢ|) / nⱼ` along the next. Scaling the first point by a
/// power of two cannot help there: scaled back it is `offset / nᵢ` again, which
/// no `f64` holds. The new one is always finite, since `(1 − |nᵢ|)/|nⱼ| ≤ 0.74`
/// for a unit normal. `1 − |nᵢ|` is exact and so is its product with the
/// offset, so the only rounding is of the second coordinate, which moves the
/// dot by at most `|nⱼ|·ulp/2 ≤ 0.354` ulp of the offset. The division only
/// overflows when `|offset| > f64::MAX/√3 > 2¹⁰²³`, the top binade, where the
/// offset's rounding interval is a full ulp wide, so the dot lands inside it;
/// and at `±f64::MAX` itself the reader's own partial sum stays finite, because
/// `nᵢ·f64::MAX` rounds down by `(1 − nᵢ)` ulp.
///
/// # Errors
///
/// [`CannotWrite::APlaneWhoseOffsetNoWrittenPointCarries`] if the point does
/// not read back as the offset after all — which is checked here, through the
/// reader's own function, rather than trusted.
fn a_point_this_reader_reads_back_as(flat: &Plane) -> Result<Point, CannotWrite> {
    let offset = flat.offset();
    let normal = plane_through(Point::origin(), flat.normal())
        .expect("a stored plane's normal is finite and has a length")
        .normal()
        .components();
    let mut widest_first = [0, 1, 2];
    widest_first.sort_by(|&a, &b| normal[b].abs().total_cmp(&normal[a].abs()));
    let [widest, next, _] = widest_first;

    let coordinates = if (offset / normal[widest]).is_finite() {
        divided_along_the_widest(offset, normal, widest, next)
    } else {
        the_offset_itself_along_the_widest(offset, normal, widest, next)
    };
    let point = Point::new(coordinates[0], coordinates[1], coordinates[2]);
    let reads_back = plane_through(point, flat.normal())
        .is_ok_and(|back| back.offset().to_bits() == offset.to_bits());
    if !reads_back {
        return Err(CannotWrite::APlaneWhoseOffsetNoWrittenPointCarries { offset });
    }
    Ok(point)
}

fn divided_along_the_widest(offset: f64, normal: [f64; 3], widest: usize, next: usize) -> [f64; 3] {
    let mut coordinates = [0.0; 3];
    coordinates[widest] = offset / normal[widest];
    let product = normal[widest] * coordinates[widest];
    let left_over = (offset - product) - normal[widest].mul_add(coordinates[widest], -product);
    if left_over != 0.0 {
        coordinates[next] = left_over / normal[next];
    }
    coordinates
}

fn the_offset_itself_along_the_widest(
    offset: f64,
    normal: [f64; 3],
    widest: usize,
    next: usize,
) -> [f64; 3] {
    let mut coordinates = [0.0; 3];
    coordinates[widest] = if normal[widest] < 0.0 {
        -offset
    } else {
        offset
    };
    let (high, low) = two_product(offset, 1.0 - normal[widest].abs());
    let quotient = high / normal[next];
    let remainder = (-quotient).mul_add(normal[next], high) + low;
    coordinates[next] = quotient + remainder / normal[next];
    coordinates
}

/// What a face sits on, for a refusal to name.
fn kind_of(surface: &Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => "plane",
        Surface::Cylinder(_) => "pipe",
        Surface::Sphere(_) => "ball",
        Surface::Cone(_) => "cone",
        Surface::Torus(_) => "ring",
        _ => "surface this writer cannot name",
    }
}

fn references(ids: &[u64]) -> String {
    ids.iter()
        .map(|id| format!("#{id}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn triple(numbers: [f64; 3]) -> String {
    format!(
        "({},{},{})",
        real(numbers[0]),
        real(numbers[1]),
        real(numbers[2])
    )
}

fn flag(yes: bool) -> &'static str {
    if yes { ".T." } else { ".F." }
}

/// A number as Part 21 writes a `REAL`.
///
/// **`{:?}` is Rust's shortest text that reads back as the same `f64`**, which
/// is the whole of why a solid written here and read back holds the same
/// numbers bit for bit rather than nearly. That is a checkable claim and
/// `every_number_this_writer_writes_reads_back_bit_for_bit` checks it, through
/// this crate's own reader, over the numbers a model is made of and the ones
/// that are hardest to print.
///
/// Two things `{:?}` does not do that Part 21 requires: a real always carries a
/// decimal point, and an exponent carries one in front of it. `1e-7` is written
/// `1.E-7`, and an integer-valued real is written `1.` and never `1` — which
/// keeps [`Value::Integer`](crate::Value) out of a file this crate wrote, so
/// [`Import::integers_read_as_reals`](crate::Import::integers_read_as_reals) is
/// zero on everything it writes.
fn real(number: f64) -> String {
    let written = format!("{number:?}");
    match written.split_once('e') {
        None => written,
        Some((mantissa, exponent)) => {
            let mantissa = if mantissa.contains('.') {
                mantissa.to_owned()
            } else {
                format!("{mantissa}.")
            };
            format!("{mantissa}E{exponent}")
        }
    }
}

/// A string as Part 21 writes one, quotes and all.
///
/// Two escapes and no more, because two is all Part 21 has for the characters a
/// caller can hand over: an apostrophe is written twice, and anything outside
/// printable ASCII is written `\X2\` followed by its UTF-16 code units and
/// closed with `\X0\`. A caller's name for its own file is a caller's bytes, so
/// this is the same rule as the reader's: whatever goes out has to come back.
fn text(string: &str) -> String {
    let mut written = String::from("'");
    let mut wide: Vec<u16> = Vec::new();
    let close = |written: &mut String, wide: &mut Vec<u16>| {
        if wide.is_empty() {
            return;
        }
        written.push_str("\\X2\\");
        for unit in wide.drain(..) {
            written.push_str(&format!("{unit:04X}"));
        }
        written.push_str("\\X0\\");
    };
    for character in string.chars() {
        if character.is_ascii() && !character.is_ascii_control() {
            close(&mut written, &mut wide);
            if character == '\'' {
                written.push('\'');
            }
            written.push(character);
        } else {
            let mut units = [0u16; 2];
            wide.extend_from_slice(character.encode_utf16(&mut units));
        }
    }
    close(&mut written, &mut wide);
    written.push('\'');
    written
}

#[cfg(test)]
mod plane_round_trip_tests {
    use core::f64::consts::TAU;

    use rcad_geom::{Axis, Isometry, Plane, Point, Vector};

    use super::{CannotWrite, a_point_this_reader_reads_back_as};
    use crate::geometry::plane_through;

    /// The point `Writing::surface` used to write, the foot of the
    /// perpendicular, read back through the reader's own function, misses the
    /// offset on many of these planes. This is what the planes below are for:
    /// a population on which the old point fails often enough that a new one
    /// passing is worth something.
    #[test]
    fn the_foot_of_the_perpendicular_misses_on_more_than_a_third_of_these_planes() {
        let planes = planes_facing_no_axis();
        let misses = planes
            .iter()
            .filter(|plane| {
                plane_through(plane.origin(), plane.normal())
                    .expect("a stored plane reads back as some plane")
                    .offset()
                    != plane.offset()
            })
            .count();
        println!(
            "the foot of the perpendicular misses {misses} of {} planes",
            planes.len()
        );
        assert!(
            misses > planes.len() / 3,
            "only {misses} of {} planes miss, so this population no longer shows \
             what the point this writer chooses is for",
            planes.len()
        );
    }

    /// Every plane on which the foot of the perpendicular misses, and every
    /// other one here, reads back from the point this writer writes with the
    /// offset it had, bit for bit.
    #[test]
    fn every_plane_is_written_at_a_point_its_offset_reads_back_from() {
        let by_hand = [
            (Vector::new(0.0, 0.0, 1.0), 5.0),
            (Vector::new(0.0, -1.0, 0.0), 0.0),
            (Vector::new(-9.0, -4.0, 5.0), 0.0),
            (Vector::new(1.0, 1.0, 0.0), 1e-300),
        ]
        .map(|(normal, offset)| {
            Plane::from_point_and_normal(Point::origin(), normal)
                .and_then(|plane| plane.moved_to(offset))
                .expect("a finite offset along a normal with a length")
        });
        for plane in planes_facing_no_axis().into_iter().chain(by_hand) {
            let point = a_point_this_reader_reads_back_as(&plane)
                .unwrap_or_else(|why| panic!("{plane:?}: {why}"));
            let back = plane_through(point, plane.normal()).expect("a finite point");
            assert_eq!(
                back.offset().to_bits(),
                plane.offset().to_bits(),
                "{plane:?} read back from {point:?} as {back:?}"
            );
        }
    }

    /// Planes so far out that `offset / nᵢ` overflows — `1.41e308 / 0.707` is
    /// the first — read back from the point this writer writes, bit for bit.
    /// Before `the_offset_itself_along_the_widest` these were refused by name.
    #[test]
    fn a_plane_too_far_out_to_divide_is_written_at_a_point_it_reads_back_from() {
        let first =
            Plane::from_point_and_normal(Point::new(1e308, 1e308, 0.0), Vector::new(1.0, 1.0, 0.0))
                .expect("an offset of 1.41e308 is still finite");
        let along = |normal: Vector, offset: f64| {
            Plane::from_point_and_normal(Point::origin(), normal)
                .and_then(|plane| plane.moved_to(offset))
                .expect("a finite offset along a normal with a length")
        };
        let others = [
            (Vector::new(1.0, 1.0, 1.0), f64::MAX),
            (Vector::new(-1.0, 1.0, 1.0), -f64::MAX),
            (Vector::new(0.6, 0.55, 0.58), f64::MAX),
            (Vector::new(1.0, 2e-8, 0.0), f64::MAX),
            (Vector::new(-3.0, 2.0, -1.0), 1.7e308),
        ]
        .map(|(normal, offset)| along(normal, offset));
        for plane in core::iter::once(first).chain(others) {
            let normal = plane_through(Point::origin(), plane.normal())
                .expect("a normal with a length")
                .normal()
                .components();
            let widest = normal.iter().map(|n| n.abs()).fold(0.0, f64::max);
            assert!(
                (plane.offset() / widest).is_infinite(),
                "{plane:?} is not one the division overflows on"
            );
            let point = a_point_this_reader_reads_back_as(&plane)
                .unwrap_or_else(|why| panic!("{plane:?}: {why}"));
            let back = plane_through(point, plane.normal()).expect("a finite point");
            assert_eq!(
                back.offset().to_bits(),
                plane.offset().to_bits(),
                "{plane:?} read back from {point:?} as {back:?}"
            );
        }
    }

    /// Planes whose offset is below `2⁻⁹⁶⁹` but not subnormal are still
    /// refused by name on some normals: there the reader rounds each product
    /// and drops what it lost (`product_is_exact`), and
    /// `divided_along_the_widest` still counts that loss. Pinned so the
    /// variant is not removed while it is reached.
    #[test]
    fn some_planes_just_above_the_subnormals_are_still_refused_by_name() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut refused = 0;
        for _ in 0..20_000 {
            let normal = Vector::new(
                (next() >> 11) as f64 / (1_u64 << 53) as f64 - 0.5,
                (next() >> 11) as f64 / (1_u64 << 53) as f64 - 0.5,
                (next() >> 11) as f64 / (1_u64 << 53) as f64 - 0.5,
            );
            let offset = f64::from_bits(next() % (50 << 52) + (1 << 52));
            let Ok(plane) = Plane::from_point_and_normal(Point::origin(), normal)
                .and_then(|plane| plane.moved_to(offset))
            else {
                continue;
            };
            match a_point_this_reader_reads_back_as(&plane) {
                Ok(_) => {}
                Err(CannotWrite::APlaneWhoseOffsetNoWrittenPointCarries { offset: named }) => {
                    assert_eq!(named.to_bits(), plane.offset().to_bits());
                    refused += 1;
                }
                Err(other) => panic!("{plane:?} refused as {other}"),
            }
        }
        println!("{refused} of 20000 planes just above the subnormals are refused");
        assert!(refused > 0, "none refused: the bottom gap may be closed");
    }

    /// The normal the writer fuzzer's `crash-0ac3e625f698733b9b143d41eb65edbf7087b463`
    /// (2026-09-26) held on a turned face: both components the `f64` just below
    /// `1/√2`. The reader moves it to the `f64` just above, and **no
    /// direction a file could hold reads back as it**.
    ///
    /// The reader's components are each the nearest `f64` to the exact unit
    /// ratio (`rcad-geom`'s `a_unit_component_is_the_nearest_f64_to_the_exact_ratio`),
    /// so a direction reads back as `(s, 0, s)` only if its unit vector is
    /// within half an ulp, `h = 2⁻⁵⁴`, of `s` in `x` and `z` and rounds to zero
    /// in `y`. The longest such vector is `(s + h, ≤2⁻¹⁰⁷⁵, s + h)`, and it is
    /// shorter than one: with `s = m·2⁻⁵³`, `2·(s + h)² < 1` is
    /// `2·(2m + 1)² < 2¹⁰⁸` in whole numbers, and the `y` term is far too
    /// small to close a gap of at least `2⁻¹⁰⁸`. So no unit vector rounds to
    /// this normal, and the writer cannot choose its way round the reader.
    #[test]
    fn no_direction_reads_back_as_a_normal_a_turn_left_short_of_length_one() {
        let long = core::f64::consts::FRAC_1_SQRT_2;
        let short = long.next_down();
        assert_eq!(
            format!("{short:?}"),
            "0.7071067811865475",
            "what the crash file held"
        );
        let read = plane_through(Point::origin(), Vector::new(short, 0.0, short))
            .expect("a direction with a length")
            .normal();
        assert_eq!(read, Vector::new(long, 0.0, long));

        let m = (short * 2_f64.powi(53)) as u128;
        assert_eq!(m as f64 * 2_f64.powi(-53), short, "s is m·2⁻⁵³ exactly");
        assert!(
            2 * (2 * m + 1).pow(2) < 1_u128 << 108,
            "the longest vector within half an ulp of (s, 0, s) reaches length one"
        );
    }

    /// Twenty thousand planes facing no axis, at offsets from `1e-300` to
    /// `1e300`: half straight from a constructor, and half turned afterwards,
    /// the way a placed solid's faces are, so that their normals sit an ulp or
    /// two off length one the way a turn leaves them.
    fn planes_facing_no_axis() -> Vec<Plane> {
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1_u64 << 53) as f64
        };
        (0..20_000)
            .map(|round| {
                let normal =
                    Vector::new(2.0 * next() - 1.0, 2.0 * next() - 1.0, 2.0 * next() - 1.0);
                let offset = (2.0 * next() - 1.0) * 10_f64.powi((next() * 600.0) as i32 - 300);
                let plane = Plane::from_point_and_normal(Point::origin(), normal)
                    .and_then(|plane| plane.moved_to(offset))
                    .expect("a finite offset along a normal with a length");
                if round % 2 == 0 {
                    return plane;
                }
                let axis = Axis::new(Point::origin(), Vector::new(next(), next(), 1.0))
                    .expect("a direction whose last component is one");
                let turn = Isometry::rotation(&axis, next() * TAU)
                    .expect("a finite angle about a unit axis");
                plane.transformed(&turn)
            })
            .collect()
    }
}

#[cfg(test)]
mod cone_round_trip_tests {
    use rcad_geom::{Axis, Cone, Point, Vector};

    /// The demonstration behind `Writing::surface`'s doc comment: a cone's
    /// slope is not written down — STEP has no field for a tangent, only a
    /// half-angle — so every cone this writer emits goes out through `atan`
    /// (here, and in `Writing::surface`) and comes back through `tan` (in
    /// `cone_from`, and here through the public
    /// [`Cone::from_half_angle`](rcad_geom::Cone::from_half_angle) that does
    /// the same conversion), and that round trip is not exact.
    #[test]
    fn a_cone_is_the_one_surface_whose_round_trip_goes_through_a_library_function() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next_u64 = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut next_f64 = |lo: f64, hi: f64| {
            let unit = (next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
            lo + unit * (hi - lo)
        };

        let axis = Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).unwrap();
        let trials = 5000;
        let mut rounds = 0;
        let mut worst_ulps: f64 = 0.0;
        for _ in 0..trials {
            let slope = next_f64(1e-3, 1e3);
            let cone = Cone::new(axis, slope).unwrap();
            let half_angle = cone.slope().atan();
            let round_tripped = Cone::from_half_angle(axis, half_angle).unwrap();
            if round_tripped.slope() != slope {
                rounds += 1;
                let slip = (round_tripped.slope() - slope).abs();
                worst_ulps = worst_ulps.max(slip / slope.abs());
            }
        }
        println!(
            "{trials} random slopes: atan-then-tan rounds {rounds} times, \
             worst relative slip {worst_ulps}"
        );
        assert!(
            rounds > trials / 10,
            "a cone's slope should round often enough that the cost in the doc \
             comment is real and not a curiosity: {rounds}"
        );
        assert!(
            worst_ulps < 1e-9,
            "the round trip should still be small, not something a caller of a \
             written-and-reread file would notice: {worst_ulps}"
        );
    }
}
