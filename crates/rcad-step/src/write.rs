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

use rcad_geom::{Point, Scale, Surface, Unit, Vector};
use rcad_topo::{EdgeKey, Model, VertexKey, WireStepView};

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
        writing.solid_from_shell(model, shell.key())?;
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
    fn solid_from_shell(
        &mut self,
        model: &Model,
        shell: rcad_topo::ShellKey,
    ) -> Result<(), CannotWrite> {
        let shell = model
            .shell(shell)
            .expect("every key here came off the model being written");
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
    /// # A general-normal plane also rounds, by 1–4 ULPs, and no point choice
    ///   here closes it
    ///
    /// STEP has no way to write a plane except as a point plus a normal, and
    /// [`Plane::from_point_and_normal`](rcad_geom::Plane::from_point_and_normal)
    /// re-derives the offset by dotting them back together on the way in. For
    /// a normal that is not axis-aligned that dot product has three nonzero
    /// terms, and a three-term dot product is not, in general, the exact
    /// inverse of the multiply that built the offset in the first place — a
    /// plane with normal `(−9,−4,5)` and offset `0` reads a point that sits
    /// exactly on it, `(0,−10,−8)`, back `4.44e-16` off itself.
    ///
    /// **Checked, not assumed, that a different point does not fix it**:
    /// [`plane_round_trip_tests::choosing_a_different_point_does_not_fix_a_general_normal_plane`]
    /// tries the obvious other candidate — a point with only one nonzero
    /// coordinate, along the normal's largest component, rather than the
    /// foot of the perpendicular this writer sends — against 5 000 random
    /// general-normal planes. It differs about as often as the point this
    /// writer already sends (in one run, 2 327 against 2 132 out of 5 000),
    /// which is not an improvement; nothing in this family of fixes is. This
    /// is a limit of STEP's own encoding of a plane, not a gap in this
    /// writer.
    ///
    /// **It does not move [`Scale::nominal_size`]** — that is set from the
    /// solid's own vertices (`Import::biggest_coordinate_in`), and a plane's
    /// placement point is not one — so the rounding here changes no
    /// published bound, only the plane's own stored coefficients by an
    /// amount `Model::what_writing_a_plane_down_costs` (`rcad-topo`) already
    /// prices in.
    fn surface(&mut self, surface: &Surface) -> Result<u64, CannotWrite> {
        Ok(match surface {
            Surface::Plane(flat) => {
                let place = self.placement(flat.origin(), flat.normal());
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
    use rcad_geom::{Plane, Point, Vector};

    /// The demonstration behind `Writing::surface`'s doc comment: a
    /// plane through the origin with normal `(−9,−4,5)` and offset `0` reads
    /// back a point that is exactly on it, `(0,−10,−8)` — the dot product is
    /// `0·(−9) + (−10)·(−4) + (−8)·5 = 0 + 40 − 40 = 0` on paper — off itself
    /// by a few ULPs.
    #[test]
    fn a_plane_through_the_origin_still_rounds_reading_its_own_point_back() {
        let normal = Vector::new(-9.0, -4.0, 5.0).normalized().unwrap();
        let point = Point::new(0.0, -10.0, -8.0);
        assert_eq!(normal.dot(point - Point::origin()), 0.0, "exact on paper");

        let plane = Plane::from_point_and_normal(point, normal).unwrap();
        assert_ne!(
            plane.offset(),
            0.0,
            "a general-normal plane's own defining point should not read back exactly"
        );
        assert!(
            plane.offset().abs() < 1e-14,
            "the rounding should still be a few ULPs, not something bigger: {}",
            plane.offset()
        );
    }

    /// Whether writing a different point — one with only one nonzero
    /// coordinate, along the normal's own largest component, rather than the
    /// foot of the perpendicular `Writing::surface` sends — reads
    /// back exactly more often. It does not: this is the evidence for "no
    /// point choice closes it" in that doc comment.
    #[test]
    fn choosing_a_different_point_does_not_fix_a_general_normal_plane() {
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
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

        let trials = 5000;
        let mut foot_of_perpendicular_rounds = 0;
        let mut single_coordinate_rounds = 0;
        for _ in 0..trials {
            let raw = Vector::new(
                next_f64(-100.0, 100.0),
                next_f64(-100.0, 100.0),
                next_f64(-100.0, 100.0),
            );
            let Ok(normal) = raw.normalized() else {
                continue;
            };
            let offset = next_f64(-1000.0, 1000.0);

            let foot = Point::origin() + normal * offset;
            if Plane::from_point_and_normal(foot, normal).unwrap().offset() != offset {
                foot_of_perpendicular_rounds += 1;
            }

            let components = normal.components();
            let (axis, &biggest) = components
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
                .unwrap();
            let mut single = [0.0; 3];
            single[axis] = offset / biggest;
            let single = Point::new(single[0], single[1], single[2]);
            if Plane::from_point_and_normal(single, normal)
                .unwrap()
                .offset()
                != offset
            {
                single_coordinate_rounds += 1;
            }
        }
        println!(
            "{trials} random general-normal planes: foot-of-perpendicular rounds \
             {foot_of_perpendicular_rounds} times, single-coordinate rounds \
             {single_coordinate_rounds} times"
        );
        assert!(
            foot_of_perpendicular_rounds > trials / 10,
            "the writer's own point choice should round often enough that a fix would show up \
             clearly: {foot_of_perpendicular_rounds}"
        );
        assert!(
            single_coordinate_rounds > trials / 10,
            "picking a different point should not make the rounding disappear: \
             {single_coordinate_rounds}"
        );
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
