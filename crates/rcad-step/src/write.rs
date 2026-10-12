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
//! line, a plane across a pipe crosses it in a circle or an ellipse. **Where those two
//! surfaces do not name a curve, it refuses by name and writes nothing**, and
//! [`CannotWrite::ACurveTheFacesEitherSideOfItDoNotName`] says what closes it.
//!
//! One edge is named before the surfaces are asked: an open edge the model
//! says runs straight is a `LINE`, whatever the faces either side, because the
//! reader reads every `LINE` back as straight. A round bar turned off `z` is
//! why: the two surfaces alone refused its seam (Q405).

use std::collections::HashMap;

use rcad_geom::{
    Cylinder, EdgeRun, HalfSpace, Isometry, NEAREST_F64_POINT_ROUNDOFF, Picker, Plane, Point,
    Scale, Sphere, Surface, Torus, Unit, Vector,
};
use rcad_predicates::expansion::two_product;
use rcad_predicates::{Decision, Point as PredicatePoint};
use rcad_topo::{
    CoedgeView, EdgeKey, FaceKey, FaceView, Model, ShellView, VertexKey, WireStepView,
};

use crate::brep::{
    A_CIRCLE_MAY_MISS_ITS_FACE_BY, a_surface_holds_the_circle, how_far_off_the_circle_can_be,
    the_curved_one_of, the_poles_of_a_ball_on_its_chart,
};
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

    /// A wire step that stands still on a corner, where this writer cannot
    /// leave the pole unwritten.
    ///
    /// An `EDGE_LOOP` is a list of `ORIENTED_EDGE`s and a pole is not one.
    /// Real files say a pole in two ways. A ball's is left unwritten: the loop
    /// runs up the seam and turns straight back down it, which is how 69
    /// `SPHERICAL_SURFACE` faces of the KiCad corpus write it
    /// (`kicad/LED_D5.0mm.step` `#17`). A cone's apex is a `VERTEX_LOOP` of its
    /// own, a second bound of the face, in 136 `CONICAL_SURFACE` faces of the
    /// NIST files (`nist_ctc_05` `#1340`). **This writer writes the first**, and
    /// the reader puts the pole back wherever a loop turns back along the edge
    /// it has just walked — so a ball comes back the ball that went out.
    ///
    /// **A whole ball turned off `z` is not refused**: its poles are not where
    /// this kernel's [`Sphere`](rcad_geom::Sphere) pinches, on the `z` line
    /// through its centre, so it is written as the same ball on that line,
    /// with the seam `add_ball` cuts, and read back as that (`docs/decisions.md`
    /// §404).
    ///
    /// **Gap, in two shapes.** A pole whose wire does not turn back along one
    /// edge there, which is a `VERTEX_LOOP` away and the reader does not walk
    /// one yet anywhere but a cone apex
    /// ([`Refusal::ALoopThisReaderDoesNotWalk`](crate::Refusal::ALoopThisReaderDoesNotWalk)).
    /// And a face that is only part of a ball, with a pole a turn moved off
    /// the `z` line: the reader would put it back where the surface does not
    /// pinch, which `Model::validate` refuses. Only the reader builds a part of
    /// a ball with a pole, where a file's loop turns back exactly on that line,
    /// so such a face is a file read in and then turned.
    ///
    /// **And one limit**: a whole ball turned off `z` whose radius is too
    /// small beside its centre's `z` for `z + r` and `z − r` to stand apart
    /// from `z`, which has no two poles on that line to write.
    #[error("a wire in this model stands still at a pole this writer cannot leave unwritten")]
    AWireThatStandsStillAtAPole,

    /// The two faces either side of an edge do not name the curve it runs
    /// along.
    ///
    /// **Gap.** The model knows exactly what this edge runs along, and
    /// `EdgeView::run` hands the located run out since `EdgeRun::transformed`
    /// landed — the function §42.5's lane reported missing for a placement
    /// argument and §42.10's for a turned arc. What closes the gap is reading
    /// it for the curved runs too, not only for straight ones and a ball's
    /// seam.
    ///
    /// This writer derives the curve from the two surfaces either
    /// side, which is what §24.3 says an edge *is*. It reads six things off
    /// the run instead. An open edge whose run is straight is a `LINE`,
    /// because the reader reads every `LINE` back as straight. A ball's
    /// seam: it runs between the two poles, which are in a line with the
    /// centre, so no plane through them is picked out by the ends, and the
    /// located run [`EdgeView::run`](rcad_topo::EdgeView::run) hands out — it is
    /// `pub` now — names the plane it was cut from; where no centre the reader
    /// holds on the ball reads back as that plane, a whole ball is written cut
    /// open where `add_ball` cuts it instead (Q1308). Which of the two arcs an
    /// open edge between a flat face and a pipe or a ball is, which the two
    /// ends do not say and the run's pickers do (Q766). The plane an open arc
    /// between two curved faces lies in, which the reader walks and which the
    /// faces cross in no single curve of their own (Q767). The plane a closed
    /// circle where a ball touches a pipe lies in, for the same reason (Q777).
    /// And the pipe an arc drawn on a flat face is named by, since the faces
    /// either side are one plane.
    ///
    /// So an arc drawn on a flat face whose run names a ball, a cone or a ring
    /// is refused here, and so is a seam of one pipe whose run names a curved
    /// surface. Both used to go out as a `LINE`, which is the chord of the arc
    /// (Q764). What closes the first is writing the circle or conic where that
    /// surface crosses the face's plane; only `SolidBuilder` has been seen to
    /// build one.
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
    ///
    /// **A pole is left unwritten where the wire turns back along the edge it
    /// has just walked**, the way a real file writes a ball, and the reader puts
    /// it back at exactly that turn. A whole ball turned so its poles leave the
    /// `z` line, or whose seam no file can carry, is written as the same ball
    /// on that line, with its seam where `add_ball` cuts it
    /// ([`a_whole_ball_written_on_its_chart`]), and so is passed over by the
    /// first pass. See
    /// [`CannotWrite::AWireThatStandsStillAtAPole`] for the poles that are
    /// refused instead.
    ///
    /// **A loop is written as the reader will store it.** The reader keeps a
    /// file's loop as the wire and, for a curved face whose `same_sense` is
    /// false, uses the face reversed (`Walk::describe_the_face`). A wire of a
    /// face used reversed comes out of `WireView::steps` walked the other way
    /// round with each edge turned, so for such a face the walk is turned back
    /// to the order the wire is stored in, each step with it. Every other face
    /// is written as walked, each edge the way the walk takes it — a plane used
    /// reversed is written already turned round, which the reader uses forwards.
    /// Writing the walk in one sense and each edge's way along it in the other
    /// wrote a loop that does not close, and the reader refused it by name.
    fn solid_from_shell(&mut self, model: &Model, shell: ShellView<'_>) -> Result<(), CannotWrite> {
        let mut either_side: HashMap<EdgeKey, Vec<Side>> = HashMap::new();
        let mut whole_balls: HashMap<FaceKey, Sphere> = HashMap::new();
        for face in shell.faces() {
            if let Some(ball) = a_whole_ball_written_on_its_chart(model, face) {
                whole_balls.insert(face.key(), ball);
                continue;
            }
            let side = Side {
                located: face.surface(),
                stored: model
                    .face(face.key())
                    .expect("a face of the model being written")
                    .surface(),
                placed_by: face.location(),
            };
            for wire in face.wires() {
                let walk: Vec<WireStepView<'_>> = wire.steps().collect();
                for (position, step) in walk.iter().enumerate() {
                    match *step {
                        WireStepView::Along(coedge) => either_side
                            .entry(coedge.edge().key())
                            .or_default()
                            .push(side),
                        WireStepView::Pole(pole)
                            if the_wire_turns_back_at(&walk, position)
                                && side
                                    .located
                                    .the_way_its_first_coordinate_runs_at(pole.point())
                                    .is_none() => {}
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
            if let Some(ball) = whole_balls.get(&face.key()) {
                let ring = self.a_whole_ball_on_its_chart(ball)?;
                let bound = self.entity(&format!("FACE_OUTER_BOUND('',#{ring},.T.)"));
                skin.push(self.entity(&format!(
                    "ADVANCED_FACE('',(#{bound}),#{sits_on},{})",
                    flag(!turned)
                )));
                continue;
            }

            let mut bounds = Vec::with_capacity(face.wire_count());
            for (which, wire) in face.wires().enumerate() {
                let mut walk: Vec<WireStepView<'_>> = wire.steps().collect();
                if turned {
                    walk.reverse();
                }
                let mut steps = Vec::with_capacity(walk.len());
                for step in walk {
                    // Every pole left here was found to be one the file can
                    // leave unwritten by the pass above.
                    let WireStepView::Along(coedge) = step else {
                        continue;
                    };
                    let key = coedge.edge().key();
                    let along_its_edge = !coedge.edge().orientation().is_reversed();
                    let edge = match edges.get(&key) {
                        Some(&already) => already,
                        None => {
                            let (start, end) = if along_its_edge {
                                (coedge.start(), coedge.end())
                            } else {
                                (coedge.end(), coedge.start())
                            };
                            let written = self.edge_curve(
                                model,
                                &mut corners,
                                (key, &coedge.edge().run()),
                                (start.key(), start.point()),
                                (end.key(), end.point()),
                                either_side.get(&key).map_or(&[][..], Vec::as_slice),
                            )?;
                            edges.insert(key, written);
                            written
                        }
                    };
                    steps.push(self.entity(&format!(
                        "ORIENTED_EDGE('',*,*,#{edge},{})",
                        flag(along_its_edge != turned)
                    )));
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

    /// One `EDGE_CURVE`, with the corners and the curve it needs. The run is
    /// the located one, where the solid is placed.
    fn edge_curve(
        &mut self,
        model: &Model,
        corners: &mut HashMap<VertexKey, u64>,
        (edge, run): (EdgeKey, &EdgeRun),
        start: (VertexKey, Point),
        end: (VertexKey, Point),
        either_side: &[Side],
    ) -> Result<u64, CannotWrite> {
        let closed = model
            .edge(edge)
            .expect("every key here came off the model being written")
            .is_closed()
            .map_err(|_| CannotWrite::ARunKindThisWriterHasNoCurveFor)?;
        let sides = match either_side {
            [first, second] => [first, second],
            _ => {
                return Err(CannotWrite::AnEdgeThatIsNotWalkedTwice {
                    walks: either_side.len(),
                });
            }
        };
        let curve = self.curve(sides, run, start.1, end.1, closed)?;
        let from = self.vertex_point(corners, start);
        let to = self.vertex_point(corners, end);
        Ok(self.entity(&format!(
            "EDGE_CURVE('',#{from},#{to},#{},{})",
            curve.id,
            flag(curve.the_edge_runs_its_way)
        )))
    }

    /// The curve an edge runs along, worked out from the two surfaces either
    /// side of it.
    ///
    /// First, before the surfaces: **an open edge whose located run is
    /// [`EdgeRun::Straight`] is a `LINE`**, whatever the faces either side. The
    /// reader turns every `LINE` back into `Straight`, so this is its rule run
    /// backwards. Without it the surfaces below refuse the seam of a round bar
    /// turned off `z` (Q405).
    ///
    /// Then three shapes, and each is the *only* curve of this kernel's vocabulary
    /// that the pair admits, which is what makes this a derivation rather than
    /// a guess:
    ///
    /// - **two planes** cross in one straight line, which the rule above has
    ///   already written. So an open edge between two flat faces that reaches
    ///   the surfaces is a curve drawn on one plane, which the plane does not
    ///   name, and it is written only where its run names a pipe — see
    ///   [`Writing::an_arc_of_a_circle_on_a_flat_face`]. **Or it is one of that
    ///   pipe's own straight lines**: where the first face's plane holds the
    ///   axis's direction, `n·a == 0.0`, the walk draws the edge straight
    ///   between its ends (`what_a_run_between_these_two_lies_on` in
    ///   `rcad-geom`), and it is written as a `LINE`. An edge between two
    ///   different planes named that way is one; it went to the circle until
    ///   Q775 and was refused by name
    ///   (`a_bar_whose_straight_edge_is_named_by_a_pipe_it_lies_along_is_written_with_a_line_and_comes_back_the_same`).
    ///   A run naming a ball, a
    ///   cone or a ring there is refused. Until Q764 it was written as a
    ///   `LINE`: a disc drawn on a bar's top with its rim named by a ball went
    ///   out as two halves of the rim written as the one chord, and the reader
    ///   refused the file
    ///   (`a_disc_drawn_on_a_bar_and_named_by_a_ball_is_refused_by_name_rather_than_written_as_a_chord`);
    /// - **a plane across a pipe** crosses it in one closed curve, centred
    ///   where the axis meets the plane: a circle where the plane is square to
    ///   the axis or so nearly square that the two sizes round to one, and an
    ///   ellipse otherwise — see [`where_a_plane_crosses_a_pipe`]. An edge
    ///   that stops short of right round is one of the two arcs its ends cut
    ///   that curve into, and which one is read off the run — see
    ///   [`Crossing::the_way_round_the_arc`];
    /// - **a seam of one pipe**, where the pipe is on both sides, joining two
    ///   corners that stand at the same place around the axis, **with a run
    ///   that names a plane**. A plane section of a cylinder that is not
    ///   parallel to the axis meets each straight line of the surface once, so
    ///   it cannot hold two points at one angle; a section parallel to the axis
    ///   is a pair of those straight lines. So of every plane section, only the
    ///   straight one joins two such corners. A seam the model calls straight
    ///   never gets here, so this is a seam named by the plane through it,
    ///   which `SolidBuilder` builds
    ///   (`a_bar_whose_seam_is_named_by_the_plane_through_its_axis_is_written_with_a_line_and_comes_back_the_same`)
    ///   and the union of a block and a bar through its floor does not: that
    ///   fuse keeps its joined wall's seams straight (Q764).
    ///   **A run naming a curved surface is refused**, because the argument
    ///   covers planes only: a pipe of radius one along `x` through
    ///   `(0, 0, 2)` crosses a pipe of radius two about `z` at `(−2, 0, 1)` and
    ///   `(−2, 0, 3)`, at one place round it, and the arc between them is no
    ///   straight line. No model has held such an edge yet. `SolidBuilder`
    ///   refused a loop drawn on a bar's wall by that pipe, as a crossing too
    ///   nearly parallel to follow, and by a ball centred on the wall or
    ///   beside it, as a crossing in two pieces it could not tell apart;
    /// - **an arc where a plane crosses a ball** is one of the two arcs its ends
    ///   cut the circle into, which [`where_a_plane_crosses_a_ball`] names and
    ///   the run picks, as for a pipe. **A closed edge there** is the whole
    ///   circle, turned the way the ball winds at its corner
    ///   ([`Crossing::it_runs_the_way_a_closed_edge_is_stored`]), and only where
    ///   its run names nothing but the two faces (Q777);
    /// - **an arc where a plane square to a ring's axis crosses it** is an arc
    ///   of one of the circles round the axis the plane crosses the ring in,
    ///   which [`where_a_plane_crosses_a_ring`] names and the run picks, as for
    ///   a pipe: the circle of the ring's own radius where the plane rests on
    ///   top of its tube, which is the one the KiCad corpus carries (Q774), and
    ///   of the two where it cuts the tube, the one the edge's corners are on
    ///   (Q778). Any other plane refuses;
    /// - **a closed edge where a flat cap crosses a cone** square to its axis
    ///   is the circle [`where_a_cap_square_to_a_cone_crosses_it`] works out in
    ///   the frame the two faces are stored in, and then places;
    /// - **a closed edge where a flat cap holding a ring's axis crosses it** —
    ///   the end of a bend of pipe — is the circle round the tube that
    ///   [`where_a_cap_holding_a_rings_axis_crosses_it`] works out the same way.
    ///
    /// And three the surfaces do not name, which are read off the run instead:
    /// **a seam of one ball from pole to pole** — see
    /// [`Writing::a_seam_of_one_ball`] — **an arc between two curved faces
    /// that lies in a plane the run names** — see
    /// [`Writing::an_arc_its_own_plane_names`] — and **a closed circle where a
    /// ball touches a pipe, in the plane the run names** — see
    /// [`Writing::a_closed_circle_where_a_ball_touches_a_pipe`].
    ///
    /// Everything else refuses by name, and
    /// [`CannotWrite::ACurveTheFacesEitherSideOfItDoNotName`] says what closes
    /// it.
    fn curve(
        &mut self,
        sides: [&Side; 2],
        run: &EdgeRun,
        start: Point,
        end: Point,
        closed: bool,
    ) -> Result<Curve, CannotWrite> {
        let [first, second] = sides.map(|side| &side.located);
        let refuse = || CannotWrite::ACurveTheFacesEitherSideOfItDoNotName {
            first: kind_of(first),
            second: kind_of(second),
            closed,
        };
        if !closed && *run == EdgeRun::Straight {
            return Ok(Curve::along(self.line(start, end - start)));
        }
        match (first, second) {
            (Surface::Plane(flat), Surface::Plane(_)) if !closed => {
                let EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Cylinder(round),
                    ..
                } = run
                else {
                    return Err(refuse());
                };
                if flat.normal().dot(round.axis().direction()) == 0.0 {
                    return Ok(Curve::along(self.line(start, end - start)));
                }
                self.an_arc_of_a_circle_on_a_flat_face(
                    flat,
                    [first, second],
                    round,
                    run,
                    start,
                    end,
                )
                .ok_or_else(refuse)
            }
            (Surface::Plane(flat), Surface::Cylinder(pipe))
            | (Surface::Cylinder(pipe), Surface::Plane(flat)) => {
                let crossing = where_a_plane_crosses_a_pipe(flat, pipe).ok_or_else(refuse)?;
                let turning = if closed {
                    crossing.turning
                } else {
                    crossing
                        .the_way_round_the_arc(run, start, end)
                        .ok_or_else(refuse)?
                };
                Ok(Curve::along(self.round(&crossing, turning)))
            }
            (Surface::Plane(flat), Surface::Sphere(ball))
            | (Surface::Sphere(ball), Surface::Plane(flat))
                if !closed || *run == EdgeRun::RightRoundWhereItsFacesCross =>
            {
                let crossing = where_a_plane_crosses_a_ball(flat, ball).ok_or_else(refuse)?;
                let turning = if closed {
                    let its_own_way = crossing
                        .it_runs_the_way_a_closed_edge_is_stored([first, second], start)
                        .ok_or_else(refuse)?;
                    if its_own_way {
                        crossing.turning
                    } else {
                        -crossing.turning
                    }
                } else {
                    crossing
                        .the_way_round_the_arc(run, start, end)
                        .ok_or_else(refuse)?
                };
                Ok(Curve::along(self.round(&crossing, turning)))
            }
            (Surface::Plane(flat), Surface::Torus(ring))
            | (Surface::Torus(ring), Surface::Plane(flat))
                if !closed =>
            {
                let crossing =
                    where_a_plane_crosses_a_ring(flat, ring, [start, end]).ok_or_else(refuse)?;
                let turning = crossing
                    .the_way_round_the_arc(run, start, end)
                    .ok_or_else(refuse)?;
                Ok(Curve::along(self.round(&crossing, turning)))
            }
            (Surface::Plane(_), Surface::Cone(_)) | (Surface::Cone(_), Surface::Plane(_))
                if closed =>
            {
                let crossing = where_a_cap_square_to_a_cone_crosses_it(sides).ok_or_else(refuse)?;
                Ok(Curve::along(self.round(&crossing, crossing.turning)))
            }
            (Surface::Plane(_), Surface::Torus(_)) | (Surface::Torus(_), Surface::Plane(_))
                if closed =>
            {
                let crossing =
                    where_a_cap_holding_a_rings_axis_crosses_it(sides, start).ok_or_else(refuse)?;
                Ok(Curve::along(self.round(&crossing, crossing.turning)))
            }
            (Surface::Cylinder(pipe), Surface::Cylinder(other)) if !closed && pipe == other => {
                let EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Plane(_),
                    ..
                } = run
                else {
                    return Err(refuse());
                };
                let along = pipe.axis().direction();
                let across = |corner: Point| {
                    let out = corner - pipe.axis().origin();
                    out - along * out.dot(along)
                };
                if across(start) != across(end) {
                    return Err(refuse());
                }
                Ok(Curve::along(self.line(start, end - start)))
            }
            (Surface::Sphere(ball), Surface::Sphere(other)) if !closed && ball == other => self
                .a_seam_of_one_ball(ball, run, start, end)
                .ok_or_else(refuse),
            (Surface::Torus(ring), Surface::Torus(other)) if !closed && ring == other => self
                .an_arc_its_own_plane_names(first, second, run, start, end)
                .ok_or_else(refuse),
            (Surface::Cylinder(_), Surface::Sphere(_))
            | (Surface::Sphere(_), Surface::Cylinder(_))
            | (Surface::Cylinder(_), Surface::Cylinder(_))
            | (Surface::Cylinder(_), Surface::Torus(_))
            | (Surface::Torus(_), Surface::Cylinder(_))
            | (Surface::Sphere(_), Surface::Sphere(_))
                if !closed =>
            {
                self.an_arc_its_own_plane_names(first, second, run, start, end)
                    .ok_or_else(refuse)
            }
            (Surface::Cylinder(pipe), Surface::Sphere(_))
            | (Surface::Sphere(_), Surface::Cylinder(pipe)) => self
                .a_closed_circle_where_a_ball_touches_a_pipe(pipe, [first, second], run, start)
                .ok_or_else(refuse),
            (Surface::Plane(_), Surface::Plane(_))
            | (Surface::Cylinder(_), Surface::Cylinder(_))
            | (Surface::Plane(_), Surface::Sphere(_))
            | (Surface::Sphere(_), Surface::Plane(_))
            | (Surface::Plane(_), Surface::Cone(_))
            | (Surface::Cone(_), Surface::Plane(_))
            | (Surface::Plane(_), Surface::Torus(_))
            | (Surface::Torus(_), Surface::Plane(_))
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

    /// The `CIRCLE` or `ELLIPSE` a plane crosses a pipe in, turning about
    /// `turning`: the crossing's own way round for a closed edge, and either
    /// way for an arc, which the way round is what picks.
    fn round(&mut self, crossing: &Crossing, turning: Vector) -> u64 {
        match crossing.shape {
            Shape::Circle { radius } => self.circle(crossing.centre, turning, radius),
            Shape::Ellipse { first_way, sizes } => {
                self.ellipse(crossing.centre, turning, first_way, sizes)
            }
        }
    }

    /// The curve a seam with the one ball on both sides runs along from
    /// `start` to `end`, written, or `None` where this writer finds none the
    /// file carries — see [`the_curve_of_a_seam_of_one_ball`].
    fn a_seam_of_one_ball(
        &mut self,
        ball: &Sphere,
        run: &EdgeRun,
        start: Point,
        end: Point,
    ) -> Option<Curve> {
        let (crossing, the_edge_runs_its_way) =
            the_curve_of_a_seam_of_one_ball(ball, run, start, end)?;
        Some(Curve {
            id: self.round(&crossing, crossing.turning),
            the_edge_runs_its_way,
        })
    }

    /// The `EDGE_LOOP` of a whole ball on the chart the reader reads every
    /// ball on: poles at `centre ± (0, 0, r)`, and the seam `add_ball` cuts,
    /// in the plane through the centre square to `y`, on the side facing `+x`.
    /// So the file is the one this writer writes for the same ball moved to
    /// that centre and not turned, byte for byte.
    ///
    /// Where `z + r` or `z − r` rounds to `z` there are no two poles on that
    /// chart, and the ball is refused at its pole as it was before it had one
    /// ([`CannotWrite::AWireThatStandsStillAtAPole`]).
    fn a_whole_ball_on_its_chart(&mut self, ball: &Sphere) -> Result<u64, CannotWrite> {
        let [north, south] = the_poles_of_a_ball_on_its_chart(ball)
            .ok_or(CannotWrite::AWireThatStandsStillAtAPole)?;
        let refuse = || CannotWrite::ACurveTheFacesEitherSideOfItDoNotName {
            first: "ball",
            second: "ball",
            closed: false,
        };
        let centre = ball.centre();
        let cut = Plane::from_point_and_normal(centre, Vector::new(0.0, 1.0, 0.0))
            .map_err(|_| refuse())?;
        let picking = HalfSpace::through_a_point_facing(centre, Vector::new(1.0, 0.0, 0.0))
            .map_err(|_| refuse())?;
        let run = EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(cut),
            picked_by: smallvec::smallvec![Picker::from(picking)],
        };
        let curve = self
            .a_seam_of_one_ball(ball, &run, north, south)
            .ok_or_else(refuse)?;
        let [from, to] = [north, south].map(|corner| {
            let at = self.cartesian_point(corner);
            self.entity(&format!("VERTEX_POINT('',#{at})"))
        });
        let seam = self.entity(&format!(
            "EDGE_CURVE('',#{from},#{to},#{},{})",
            curve.id,
            flag(curve.the_edge_runs_its_way)
        ));
        let there = self.entity(&format!("ORIENTED_EDGE('',*,*,#{seam},.T.)"));
        let back = self.entity(&format!("ORIENTED_EDGE('',*,*,#{seam},.F.)"));
        Ok(self.entity(&format!("EDGE_LOOP('',(#{there},#{back}))")))
    }

    /// The arc between two curved faces that lies in the plane its run names,
    /// or `None` where the run names no such plane or the plane would not
    /// read back as itself.
    ///
    /// **The reader names such an arc by the plane the file's curve lies in**,
    /// once it has checked that every face holds the curve
    /// (`Walk::the_surface_that_names_the_curve`), and then walks where the
    /// plane crosses one of the faces (`which_two_surfaces_a_run_is_between`).
    /// So the plane the run carries *is* walked, unlike the plane beside an arc
    /// of a flat face, and it has to come back bit for bit: the curve is
    /// written in it, about its own normal, at a centre from which the reader's
    /// [`plane_through`] gives back exactly that plane — which is asked of that
    /// function here rather than trusted.
    ///
    /// **The curve is where the plane crosses a pipe along the edge, or else a
    /// ball** ([`where_a_plane_crosses_a_pipe`], [`where_a_plane_crosses_a_ball`]):
    /// the curve the reader will check every face holds. A ball whose centre
    /// is on a pipe's axis, two pipes of one radius whose axes cross, and a
    /// ball's seam that stops short of a pole all meet in such a curve (Q767),
    /// and so do a pipe and a ring about one axis, and a pipe that carries on
    /// where a ring's tube stops (Q773).
    ///
    /// **Which arc is the way the edge runs along the curve**: the plane is
    /// written facing the way it faces, so the arc the run picks is chosen by
    /// `EDGE_CURVE.same_sense` rather than by turning the curve over, which
    /// would read back as the plane reversed.
    ///
    /// **The centre is one both faces hold the circle at**, where a pipe or a
    /// ball is beside the edge with another face
    /// ([`a_centre_for_an_arc_between`]): the reader asks every face before it
    /// names such an arc by its plane, and the walk alone can land a dome's
    /// rim further off the ball than it allows (Q1792). **A seam with one ring on
    /// both sides may have its centre further out** than
    /// [`a_centre_its_plane_reads_back_from`] steps, and then
    /// [`the_nearest_centre_its_faces_hold_the_circle_at`] looks as far as the
    /// reader would still hold the circle on the ring.
    fn an_arc_its_own_plane_names(
        &mut self,
        first: &Surface,
        second: &Surface,
        run: &EdgeRun,
        start: Point,
        end: Point,
    ) -> Option<Curve> {
        let (crossing, the_edge_runs_its_way) =
            the_arc_its_own_plane_names(first, second, run, start, end)?;
        Some(Curve {
            id: self.round(&crossing, crossing.turning),
            the_edge_runs_its_way,
        })
    }

    /// The closed circle a ball touches a pipe round, in the plane its run
    /// names, or `None` where the run names no plane or the circle would not
    /// read back on both faces.
    ///
    /// **A ball touches a pipe round a circle only where its centre is on the
    /// pipe's axis and the two radii are one**, and then it does not cross the
    /// pipe at all (§124). So the two faces name no curve, and the run carries
    /// the plane the circle lies in, square to the axis through the ball's
    /// centre: [`EdgeRun::RightRoundWhereItCrosses`], the kind a closed edge
    /// takes when the faces either side name nothing.
    ///
    /// **The reader cuts such a circle in two at its far point** and walks each
    /// half in the plane the file's circle lies in, once it has checked that
    /// both faces hold the circle (`Walk::describe_the_halves_of_a_circle`).
    /// So the circle is written as [`Writing::an_arc_its_own_plane_names`]
    /// writes an arc, in that plane and about its normal, at a centre the
    /// reader's [`plane_through`] reads the plane back from; and only where
    /// [`a_surface_holds_the_circle`], shared with the reader, says both faces
    /// hold it, since elsewhere the reader names the edge by its faces, which
    /// only touch, and refuses.
    ///
    /// **The way round is `same_sense`**, and the plane keeps the way it faces:
    /// the halves run the way the file's curve runs, so the edge runs along the
    /// circle where the circle turns at the corner the way the model stores the
    /// edge running ([`Crossing::it_runs_the_way_a_closed_edge_is_stored`]).
    fn a_closed_circle_where_a_ball_touches_a_pipe(
        &mut self,
        pipe: &Cylinder,
        either_side: [&Surface; 2],
        run: &EdgeRun,
        corner: Point,
    ) -> Option<Curve> {
        let EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(cut),
        } = run
        else {
            return None;
        };
        let touching = where_a_plane_crosses_a_pipe(cut, pipe)?;
        let Shape::Circle { radius } = touching.shape else {
            return None;
        };
        let centre = a_centre_its_plane_reads_back_from(cut, touching.centre, |_| true)?;
        if !either_side
            .iter()
            .all(|face| a_surface_holds_the_circle(face, centre, cut.normal(), radius))
        {
            return None;
        }
        let crossing = Crossing {
            centre,
            turning: cut.normal(),
            shape: Shape::Circle { radius },
        };
        let the_edge_runs_its_way =
            crossing.it_runs_the_way_a_closed_edge_is_stored(either_side, corner)?;
        Some(Curve {
            id: self.round(&crossing, crossing.turning),
            the_edge_runs_its_way,
        })
    }

    /// An arc of a circle drawn on a flat face, which the run names by the pipe
    /// of the circle's radius about its axis; `None` where the faces along it
    /// would not hold that circle, the pipe would not read back with its own
    /// axis and radius, or the run's pickers do not tell the two arcs apart.
    ///
    /// **Two flat faces either side of an edge that bends are one plane**, and
    /// a plane crossed with itself names nothing, so the reader names such an
    /// arc by that pipe (`Walk::the_surface_that_names_the_curve`) and walks
    /// where it crosses the face. The pipe is written as a `CIRCLE` of its
    /// radius about its axis, and the reader's `Axis::as_written` and
    /// `Cylinder::new` are asked here whether they give that axis and radius
    /// back. Until Q1001 both sides asked `Axis::new`, which scales a turned axis
    /// again and can move its last bit, so a disc drawn on a bar's top and
    /// turned was refused by name wherever it did: 76 of the 372 turns in
    /// `how_often_a_turned_disc_drawn_on_a_bar_comes_back_from_its_own_file`
    /// (`docs/decisions.md` §355.6). Writing the chord
    /// instead, as this writer did for every open edge between two flat
    /// faces, drew a face whose edge cut through its own holes: twenty KiCad
    /// LEDs, once Q767 let them past their domes, came back refused as
    /// `BoundaryCrossesItself`.
    ///
    /// **The circle is centred where the faces hold it.** The reader builds the
    /// pipe through the circle's centre, and names the run by it only where
    /// every face along the edge holds the circle ([`a_surface_holds_the_circle`],
    /// shared with the reader); otherwise it names the run by the circle's own
    /// plane, and refuses the file. The pipe's own axis origin is written where
    /// the faces hold the circle there, which brings the pipe back bit for bit.
    /// Anywhere else on the axis it is not: the pipe about a bar's own axis,
    /// whose origin is on the floor, drew a disc on the top whose circle went
    /// out on the floor, and the reader refused the file (Q775,
    /// `a_disc_drawn_on_a_bar_by_the_pipe_about_its_own_axis_comes_back_the_same`).
    /// So there the circle goes where the axis meets `flat`, worked out the way
    /// `rcad_geom::walk` centres a plane's crossing with a pipe
    /// ([`where_a_plane_crosses_a_pipe`]), which makes it the circle the model's
    /// edge is walked along. **That point rounds**, so the pipe read back runs
    /// through it and not along the model's line, and the reader's own walk of
    /// it can land a few ulps away. The check above bounds how far: the faces
    /// hold the circle only where its centre is within the reader's allowance of
    /// them and its axis square to them to within it, and the walk moves the
    /// centre along the axis by how far off the face it is.
    /// `every_turned_disc_drawn_by_a_pipe_off_its_top_comes_back_from_its_own_file`
    /// holds the turns.
    fn an_arc_of_a_circle_on_a_flat_face(
        &mut self,
        flat: &Plane,
        either_side: [&Surface; 2],
        round: &Cylinder,
        run: &EdgeRun,
        start: Point,
        end: Point,
    ) -> Option<Curve> {
        let axis = round.axis().direction();
        let radius = round.radius();
        let normal = plane_through(Point::origin(), axis).ok()?.normal();
        let held = |centre: Point| {
            either_side
                .iter()
                .all(|face| a_surface_holds_the_circle(face, centre, normal, radius))
        };
        let origin = round.axis().origin();
        let centre = if held(origin) {
            origin
        } else {
            where_a_plane_crosses_a_pipe(flat, round)?.centre
        };
        if !held(centre) {
            return None;
        }
        let read_back = rcad_geom::Axis::as_written(centre, axis)
            .and_then(|line| Cylinder::new(line, radius))
            .ok()?;
        if read_back.axis().direction() != axis || read_back.radius() != radius {
            return None;
        }
        let crossing = Crossing {
            centre,
            turning: axis,
            shape: Shape::Circle { radius },
        };
        let the_edge_runs_its_way = crossing.the_arc_runs_its_own_way(run, start, end)?;
        Some(Curve {
            id: self.circle(centre, axis, radius),
            the_edge_runs_its_way,
        })
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

    /// An `ELLIPSE` whose first size runs along `first_way`, so its placement
    /// carries that as its reference direction: without one, which way the
    /// long size runs would be a default the schema picks.
    fn ellipse(&mut self, centre: Point, axis: Vector, first_way: Vector, sizes: [f64; 2]) -> u64 {
        let at = self.cartesian_point(centre);
        let way = self.direction(axis);
        let reference = self.direction(first_way);
        let place = self.entity(&format!("AXIS2_PLACEMENT_3D('',#{at},#{way},#{reference})"));
        self.entity(&format!(
            "ELLIPSE('',#{place},{},{})",
            real(sizes[0]),
            real(sizes[1])
        ))
    }

    /// One of the five surfaces this kernel holds, in the parametrisation STEP
    /// asks for.
    ///
    /// Four of the five are written in the numbers the kernel already stores
    /// and read back as those numbers. **The cone is not**, and that is worth
    /// stating: STEP writes a half-angle and [`Cone`](rcad_geom::Cone) stores
    /// its tangent, so a cone goes out as an angle and comes back through
    /// `tan`, which Rust does not bound. The apex is exact — a placement at
    /// the apex has radius zero and `cone_from` reads that back as the origin
    /// the file wrote, with no division. The slope is written as the angle
    /// [`a_half_angle_that_reads_back_as`] finds, whose tangent is the slope
    /// itself wherever an angle near `atan` has one, and for most slopes over
    /// one none does (`docs/decisions.md` §366).
    /// `a_cone_is_the_one_surface_whose_round_trip_goes_through_a_library_function`
    /// measures what `atan` alone would cost.
    ///
    /// # A plane is written at a point its offset reads back from exactly
    ///
    /// STEP has no way to write a plane except as a point and a normal, and
    /// the reader dots the two back together to get the offset
    /// ([`plane_through`]), so the point decides the offset's last bit. The
    /// foot of the perpendicular, which this writer used to send, rounds each
    /// of its three coordinates and misses on 9 271 of 20 000 planes that face
    /// no axis (`the_foot_of_the_perpendicular_misses_on_more_than_a_third_of_these_planes`;
    /// 7 777 while the reader still scaled every normal it read).
    /// [`a_point_this_reader_reads_back_as`] picks one that does not miss
    /// (`docs/decisions.md` §313).
    ///
    /// **The normal is written as the model stores it, and the reader keeps
    /// it.** A normal a turn left an ulp or two off length one is within
    /// `Plane::NORMAL_LENGTH_ROUNDOFF` of it, and [`plane_through`] stores such
    /// a direction as written rather than scale it again. Scaling it was what
    /// moved it: `no_direction_reads_back_as_a_normal_a_turn_left_short_of_length_one`
    /// shows one that no direction reads back as once scaled, which is why no
    /// choice this writer made could have helped (§313.3).
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
                    real(a_half_angle_that_reads_back_as(cone.slope()))
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
    /// The third attribute is optional in the schema and nothing written
    /// through here needs one — an arc is picked by its two ends and its axis,
    /// never by where the parametrisation starts — so it is written `$` rather
    /// than filled in with a direction nothing chose. An ellipse does need one,
    /// and [`Writing::ellipse`] writes its own.
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

/// The located ball `face` covers whole, where it is written on the world's
/// chart rather than cut open where it is, or `None`.
///
/// A whole ball is one wire: a seam with two corners, a pole, the seam walked
/// back, a pole. Where the face is stored both poles stand where the stored
/// ball pinches, so the face is the whole ball, and it covers the whole sphere
/// wherever its seam runs. So the file need not say where the seam is, and two
/// kinds are written as the same ball cut open where `add_ball` cuts it
/// (`docs/decisions.md` §404):
///
/// - **One whose placement turned its poles off the `z` line** the located
///   ball pinches on. This kernel's [`Sphere`] carries no poles of its own
///   and pinches on the world's `z` line wherever it is put, so a turned
///   ball's poles cannot be written where its placement put them: the reader
///   would put them back where its ball does not pinch, which
///   `Model::validate` refuses.
/// - **One whose seam no file carries**, where
///   [`the_curve_of_a_seam_of_one_ball`] finds no curve. A turn about `z`
///   leaves the poles on the line, and such a ball is written as it is, seam
///   and all, wherever the reader can be given the seam's plane to the bit.
///   It is given it only through the centre of the circle the seam is half
///   of, and the reader holds that circle on the ball only with its centre a
///   few ulps from the ball's. Where the plane passes near the origin and the
///   ball does not, one ulp of a coordinate moves the offset by many ulps of
///   it, and no centre the reader holds may read back as the offset at all
///   (`no_centre_the_reader_holds_on_the_ball_carries_the_seam_of_a_ball_turned_149_degrees`).
///   Those balls were refused by name (Q1308).
fn a_whole_ball_written_on_its_chart(model: &Model, face: FaceView<'_>) -> Option<Sphere> {
    let Surface::Sphere(ball) = face.surface() else {
        return None;
    };
    let stored = model
        .face(face.key())
        .expect("a face of the model being written");
    let (_, a_whole_ball) = the_seam_of_a_ball_cut_open_once(stored)?;
    let (seam, poles_on_the_line) = the_seam_of_a_ball_cut_open_once(face)?;
    if !a_whole_ball {
        return None;
    }
    if !poles_on_the_line {
        return Some(ball);
    }
    let [start, end] = if seam.edge().orientation().is_reversed() {
        [seam.end().point(), seam.start().point()]
    } else {
        [seam.start().point(), seam.end().point()]
    };
    the_curve_of_a_seam_of_one_ball(&ball, &seam.edge().run(), start, end)
        .is_none()
        .then_some(ball)
}

/// The seam of the ball `on` is cut open along, and whether both its poles
/// stand where the ball pinches, or `None` where `on` is not one wire walking
/// one edge of two corners there and back with a pole at each turn.
fn the_seam_of_a_ball_cut_open_once(on: FaceView<'_>) -> Option<(CoedgeView<'_>, bool)> {
    let mut wires = on.wires();
    let (Some(wire), None) = (wires.next(), wires.next()) else {
        return None;
    };
    let walk: Vec<WireStepView<'_>> = wire.steps().collect();
    let [first, second, third, fourth] = walk[..] else {
        return None;
    };
    let (there, back, poles) = match (first, second, third, fourth) {
        (
            WireStepView::Along(there),
            WireStepView::Pole(one),
            WireStepView::Along(back),
            WireStepView::Pole(other),
        )
        | (
            WireStepView::Pole(one),
            WireStepView::Along(there),
            WireStepView::Pole(other),
            WireStepView::Along(back),
        ) => (there, back, [one.point(), other.point()]),
        _ => return None,
    };
    if there.edge().key() != back.edge().key() || there.start().key() == there.end().key() {
        return None;
    }
    let poles_pinch = poles.iter().all(|pole| {
        on.surface()
            .the_way_its_first_coordinate_runs_at(*pole)
            .is_none()
    });
    Some((there, poles_pinch))
}

/// The curve a seam with the one ball on both sides runs along from `start`
/// to `end`, and whether the edge runs along it the curve's own way, or `None`
/// where this writer finds none the file carries: half of a great circle from
/// pole to pole ([`the_half_circle_from_pole_to_pole`]), or else an arc in the
/// plane its run names ([`the_arc_its_own_plane_names`]).
///
/// [`Writing::a_seam_of_one_ball`] writes it, and
/// [`a_whole_ball_written_on_its_chart`] asks it first, so that a whole ball
/// is written on its chart wherever its own seam would be refused here.
fn the_curve_of_a_seam_of_one_ball(
    ball: &Sphere,
    run: &EdgeRun,
    start: Point,
    end: Point,
) -> Option<(Crossing, bool)> {
    let skin = Surface::Sphere(*ball);
    the_half_circle_from_pole_to_pole(ball, run, start, end)
        .or_else(|| the_arc_its_own_plane_names(&skin, &skin, run, start, end))
}

/// The great circle a ball's seam runs half of from `start` to `end`, and
/// whether the seam runs round it the way it turns, or `None` where this is
/// not one.
///
/// **Only between the two poles**, where the ball has no way round
/// ([`Surface::the_way_its_first_coordinate_runs_at`]), which is exactly on
/// the `z` line through the centre. A plane through both poles holds that
/// line and so the centre, so the seam is half of the circle of the ball's
/// own centre and radius in the plane the run was cut from. An edge with
/// the one ball on both sides and its ends anywhere else could be an arc of
/// a smaller circle, and is refused rather than written as a great one.
///
/// **Which half is the way the edge runs along the circle.** A `CIRCLE`
/// runs the way a right hand turns about its axis `w`, and the reader names
/// the arc from `A` to `B` by the half-space facing `(B − A) × w`, which
/// every point of that arc is strictly in front of, or facing the other way
/// where `EDGE_CURVE.same_sense` is false (`Walk::the_arc_between`). It
/// names the run by the plane about `w`, so `w` is the plane's own normal
/// and the edge runs along the circle where `(B − A) × w` faces the side
/// the run's own half-space faces, and against it where it faces away.
/// Turned over to about the reversed normal instead, the circle picked the
/// same half and came back with the seam's plane reversed, which measured a
/// different volume (Q776). The sign is read in plain `f64`: the two
/// directions are along each other or opposite, never square, so a dot
/// that comes out zero is a run this does not recognise and is refused.
///
/// **The circle is centred where the reader's plane comes back as the
/// run's** ([`a_centre_its_plane_reads_back_from`]). The reader names the
/// seam by the plane through the circle's centre about its axis, and the
/// ball's own centre, dotted with a normal a turn left, can land an ulp off
/// the offset the turn left (Q822). That centre can be fifteen `f64`s of one
/// coordinate off the ball's, not an ulp or two: every point of the circle
/// moves with it, so the circle can sit `15ε` of the largest number off the
/// ball, and the reader allows sixteen. Only measurement says the roundings
/// fit in the one left (Q1789). Where no centre is found this is `None`, and
/// a whole ball is then written on its chart
/// ([`a_whole_ball_written_on_its_chart`]).
fn the_half_circle_from_pole_to_pole(
    ball: &Sphere,
    run: &EdgeRun,
    start: Point,
    end: Point,
) -> Option<(Crossing, bool)> {
    let skin = Surface::Sphere(*ball);
    let at_a_pole = |corner: Point| skin.the_way_its_first_coordinate_runs_at(corner).is_none();
    if start == end || !at_a_pole(start) || !at_a_pole(end) {
        return None;
    }
    let EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cut),
        picked_by,
    } = run
    else {
        return None;
    };
    let [Picker::InFrontOf(picking)] = picked_by.as_slice() else {
        return None;
    };
    let leaning = (end - start).cross(cut.normal()).dot(picking.normal());
    let the_edge_runs_its_way = if leaning > 0.0 {
        true
    } else if leaning < 0.0 {
        false
    } else {
        return None;
    };
    let centre = a_centre_its_plane_reads_back_from(cut, ball.centre(), |_| true)?;
    let circle = Crossing {
        centre,
        turning: cut.normal(),
        shape: Shape::Circle {
            radius: ball.radius(),
        },
    };
    Some((circle, the_edge_runs_its_way))
}

/// What [`Writing::an_arc_its_own_plane_names`] writes, and whether the edge
/// runs along it the curve's own way, worked out without writing anything.
fn the_arc_its_own_plane_names(
    first: &Surface,
    second: &Surface,
    run: &EdgeRun,
    start: Point,
    end: Point,
) -> Option<(Crossing, bool)> {
    let EdgeRun::PartOfWhereItCrosses { and, .. } = run else {
        return None;
    };
    let Surface::Plane(cut) = and else {
        return None;
    };
    if and == first || and == second {
        return None;
    }
    let a_pipe = |face: &Surface| {
        if let Surface::Cylinder(pipe) = face {
            Some(*pipe)
        } else {
            None
        }
    };
    let a_ball = |face: &Surface| {
        if let Surface::Sphere(ball) = face {
            Some(*ball)
        } else {
            None
        }
    };
    let a_ring = |face: &Surface| {
        if let Surface::Torus(ring) = face {
            Some(*ring)
        } else {
            None
        }
    };
    let (crossing, centre) = if let Some(pipe) = a_pipe(first).or_else(|| a_pipe(second)) {
        let crossing = where_a_plane_crosses_a_pipe(cut, &pipe)?;
        let centre = a_centre_for_an_arc_between(first, second, cut, &crossing)?;
        (crossing, centre)
    } else if let Some(ball) = a_ball(first).or_else(|| a_ball(second)) {
        let crossing = where_a_plane_crosses_a_ball(cut, &ball)?;
        let centre = a_centre_for_an_arc_between(first, second, cut, &crossing)?;
        (crossing, centre)
    } else {
        let ring = a_ring(first).or_else(|| a_ring(second))?;
        let crossing = where_a_plane_square_to_a_ring_crosses_it_nearest(cut, &ring, start)?;
        let centre =
            a_centre_its_plane_reads_back_from(cut, crossing.centre, |_| true).or_else(|| {
                the_nearest_centre_its_faces_hold_the_circle_at(
                    cut,
                    &[&Surface::Torus(ring)],
                    &crossing,
                )
            })?;
        (crossing, centre)
    };
    let crossing = Crossing {
        centre,
        turning: cut.normal(),
        shape: crossing.shape,
    };
    let the_edge_runs_its_way = crossing.the_arc_runs_its_own_way(run, start, end)?;
    Some((crossing, the_edge_runs_its_way))
}

/// Where to centre the circle of an arc between `first` and `second`, a pipe
/// or a ball beside the edge, so that the reader names the arc by `cut`: a
/// point from which [`plane_through`] gives back `cut`, and where both faces
/// hold the circle as the reader asks it ([`a_surface_holds_the_circle`],
/// shared with the reader).
///
/// **The walk asks both faces, then the wider search does**, then the walk is
/// taken without asking. A dome on a long bar, turned, can have the walk land
/// on the offset more than the reader's sixteen units off the ball, and the
/// reader then names the rim by the pipe and the ball and comes back with a
/// different volume, or, for half a ball of the bar's own radius, which only
/// touches the bar, refuses the file (Q1792,
/// `a_dome_on_a_long_bar_turned_a_long_way_comes_back_with_its_rim_cut_from_its_plane`).
///
/// **Where neither finds a centre both faces hold, the walk's own is
/// written**, as it was before both were asked. A dome wider than its bar is
/// then read back named by its faces, which cross: sound, but not the solid
/// that was written. Turned and placed far out, one measured `13968.30`
/// built and `13974.48` read back. That is a defect of its own, left to a
/// row of its own (Q1792); writing the walk's centre keeps it as it was.
///
/// **A seam, one face on both sides, and an ellipse are not asked**: the
/// reader names a seam by the plane its run carries without asking the face
/// (`Walk::the_surface_that_names_the_curve` asks only where
/// `a_face_where_curved_faces_meet` names a face, and faces that are all one
/// surface name none), and holds an ellipse by a rule in `brep.rs` this
/// writer does not share.
fn a_centre_for_an_arc_between(
    first: &Surface,
    second: &Surface,
    cut: &Plane,
    crossing: &Crossing,
) -> Option<Point> {
    let Shape::Circle { radius } = crossing.shape else {
        return a_centre_its_plane_reads_back_from(cut, crossing.centre, |_| true);
    };
    if first == second {
        return a_centre_its_plane_reads_back_from(cut, crossing.centre, |_| true);
    }
    let both_hold = |centre: Point| {
        [first, second]
            .iter()
            .all(|face| a_surface_holds_the_circle(face, centre, cut.normal(), radius))
    };
    a_centre_its_plane_reads_back_from(cut, crossing.centre, both_hold)
        .or_else(|| {
            the_nearest_centre_its_faces_hold_the_circle_at(cut, &[first, second], crossing)
        })
        .or_else(|| a_centre_its_plane_reads_back_from(cut, crossing.centre, |_| true))
}

/// Whether the pole at `position` of `walk` stands where the wire turns back
/// along the edge it has just walked: the step before it walks an edge with
/// two corners into the pole, and the step after walks that same edge straight
/// back out. That is the turn the reader puts a pole back at, and no other.
fn the_wire_turns_back_at(walk: &[WireStepView<'_>], position: usize) -> bool {
    let before = walk[(position + walk.len() - 1) % walk.len()];
    let after = walk[(position + 1) % walk.len()];
    let (WireStepView::Along(there), WireStepView::Along(back)) = (before, after) else {
        return false;
    };
    there.edge().key() == back.edge().key()
        && there.start().key() != there.end().key()
        && there.end().key() == back.start().key()
}

/// A curve written under an edge, and whether the edge runs along it the way
/// the curve itself runs — `EDGE_CURVE.same_sense`.
struct Curve {
    id: u64,
    the_edge_runs_its_way: bool,
}

impl Curve {
    fn along(id: u64) -> Self {
        Self {
            id,
            the_edge_runs_its_way: true,
        }
    }
}

/// One face either side of an edge: the surface it sits on where its solid is
/// placed, and the same surface in the frame the face is stored in, with the
/// location that places it.
///
/// **The stored frame is where a builder's exact numbers still are.** A cap a
/// builder put square to its cone is square there bit for bit, and a turn
/// rounds the cap's normal and the cone's axis apart by an ulp. A curve worked
/// out there and then placed is the curve the builder meant, moved; one worked
/// out from the placed surfaces asks a question the rounding has already
/// answered wrong. A cap on a ring is asked there too (Q407).
#[derive(Clone, Copy)]
struct Side {
    located: Surface,
    stored: Surface,
    placed_by: Isometry,
}

/// The closed curve a plane crosses a pipe, a ball or a ring in.
struct Crossing {
    /// Where the pipe's or the ring's axis meets the plane, or the foot of the
    /// ball's centre on it.
    centre: Point,
    /// The way a closed edge runs round it, which is the way `rcad-topo`
    /// stores one: the pipe's axis where the plane is exactly square to it,
    /// and otherwise the plane's normal turned to face the way the axis does.
    turning: Vector,
    shape: Shape,
}

enum Shape {
    Circle {
        radius: f64,
    },
    /// The first size runs along `first_way`, which lies in the plane and is
    /// square to the pipe's axis.
    Ellipse {
        first_way: Vector,
        sizes: [f64; 2],
    },
}

/// The closed curve a plane crosses a pipe in, or `None` where the plane
/// runs along the pipe and crosses it in no closed curve at all, and where it
/// so nearly does that the curve is longer than any `f64`.
///
/// With `n` the plane's normal and `a` the pipe's axis, the curve is centred
/// where the axis meets the plane. It is a circle of the pipe's radius `r`
/// where `n` and `a` are one line, and an ellipse in the plane otherwise:
/// `r` along `n × a`, which is square to the axis, and `r / cos θ` across
/// that, with `θ` the angle between `n` and `a`.
///
/// **That second size is `r·√(1 + tan²θ)`, with `tan θ = |n × a| / |n·a|`**,
/// which is the same number whatever lengths `n` and `a` have. `cos θ`
/// worked out from `n·a` alone would not be: a normal a turn left an ulp off
/// length one moves the dot by an ulp, and that ulp would be the only thing
/// telling the two sizes apart.
///
/// **A plane a few ulps off square to its pipe crosses it in a circle in
/// `f64`**, and that is where real files have it: KiCad's round bars carry
/// caps whose normal is `1e-16` off the axis (Q763), and a bar can be built
/// whose axis is off its cap's normal by a subnormal (Q409). Under a tilt of
/// about `1e-8` the long size rounds to `r`, so the two sizes are one
/// number and the curve is the circle of radius `r` **in the plane, about
/// its normal** — not about the pipe's axis, whose circle stands up off the
/// plane by up to `r·sin θ`.
///
/// Where the two sizes differ the curve is an `ELLIPSE`, its first size
/// along `n × a`. The reader names a closed edge between a flat face and a
/// curved one by where the faces cross, and an arc by those faces and the
/// curve's own plane, so it reads the curve's centre and axis back but never
/// its sizes.
///
/// **A long size past `f64::MAX` is a limit, not a gap** (`docs/decisions.md`
/// §16.10), and it takes a plane within about `r / f64::MAX` of running along
/// the pipe. `add_bar_cut_off_by` builds a bar of radius 2 whose caps lean
/// `2^-1023` off that, so `tan θ` is `2^1023` and the long size `2^1024`: the
/// cap meets the pipe's skin `2^1024` along it, which is no point an `f64`
/// holds, and `Model::validate` refuses that bar
/// (`a_bar_whose_caps_almost_run_along_it_is_refused_by_name_because_their_ellipses_outgrow_f64`).
fn where_a_plane_crosses_a_pipe(flat: &Plane, pipe: &Cylinder) -> Option<Crossing> {
    let along = pipe.axis().direction();
    let facing = flat.normal().dot(along);
    if facing == 0.0 {
        return None;
    }
    let from = pipe.axis().origin();
    let reached = (flat.offset() - flat.normal().dot(from - Point::origin())) / facing;
    let centre = from + along * reached;
    let radius = pipe.radius();
    let across = flat.normal().cross(along);
    if across == Vector::zero() {
        return Some(Crossing {
            centre,
            turning: along,
            shape: Shape::Circle { radius },
        });
    }
    let turning = if facing > 0.0 {
        flat.normal()
    } else {
        -flat.normal()
    };
    let leaning = across.length() / facing.abs();
    let long = if leaning <= 1.0 {
        radius * leaning.mul_add(leaning, 1.0).sqrt()
    } else {
        radius * leaning * (1.0 / leaning).mul_add(1.0 / leaning, 1.0).sqrt()
    };
    if long == radius {
        return Some(Crossing {
            centre,
            turning,
            shape: Shape::Circle { radius },
        });
    }
    if !long.is_finite() {
        return None;
    }
    Some(Crossing {
        centre,
        turning,
        shape: Shape::Ellipse {
            first_way: across,
            sizes: [radius, long],
        },
    })
}

/// The circle a plane crosses a ball in, about the plane's normal, or `None`
/// where the plane misses the ball or only touches it.
///
/// Centred at the foot of the ball's centre on the plane, with radius
/// `√((ρ − h)(ρ + h))` for a ball of radius `ρ` whose centre is `h` off the
/// plane. Where `h` is zero the radius is the ball's own, with no rounding: a
/// dome on a flat face and a ball's seam both have their plane through the
/// centre.
fn where_a_plane_crosses_a_ball(flat: &Plane, ball: &Sphere) -> Option<Crossing> {
    let off = flat.signed_distance_to(ball.centre());
    let reach = ball.radius();
    if !off.is_finite() || off.abs() >= reach {
        return None;
    }
    let radius = if off == 0.0 {
        reach
    } else {
        ((reach - off.abs()) * (reach + off.abs())).sqrt()
    };
    Some(Crossing {
        centre: ball.centre() - flat.normal() * off,
        turning: flat.normal(),
        shape: Shape::Circle { radius },
    })
}

/// The circle round its axis a plane square to a ring crosses it in, the one
/// both `corners` are on, about the plane's normal; `None` for every other
/// plane, and where the corners do not say which circle.
///
/// **A plane square to the ring's axis crosses it in circles about the axis**,
/// centred where the axis meets the plane. As far along the axis from the
/// centre as the tube's radius `r`, the plane rests on the tube and touches the
/// ring all the way round one circle, of the ring's own radius `R`. A flat face
/// resting on a bend like that is the one crossing of a plane and a ring the
/// KiCad corpus carries: the eight such edges of the two
/// `Relay_1-Form-*_Schrack-RYII_RM5mm` relays and
/// `Relay_SPDT_Finder_34.51_Vertical` are each a quarter of that circle
/// (`the_edges_where_a_relay_pin_rests_on_a_ring_are_quarters_of_its_own_circle`).
/// Nearer the centre than `r`, at `h` along the axis, the plane crosses the
/// tube in two circles, of radii `R ± √(r² − h²)`: the top of a bent rod sawn
/// off flat meets its ring along both
/// (`the_top_of_a_bent_rod_sawn_off_flat_is_written_with_both_circles_the_cut_crosses_its_ring_in`).
///
/// **The circles are worked out the way `rcad_geom::walk` works them out**, so
/// that this writes the circle the walk draws for the pair either side: square
/// where `n × w` is exactly zero, resting where `r² − h²` is not positive, and
/// the two radii `R` plus and minus the root of it
/// (`circles_round_the_axis` in `crates/rcad-geom/src/walk.rs`). The reader
/// reads none of this back but the circle's axis, which with `same_sense`
/// picks the arc, so a centre or a radius rounded differently from the walk's
/// would still read back as the same edge; it is the walk's own circle so that
/// what the file says is where the edge is. The radius is not idle here,
/// though: [`Crossing::the_way_round_the_arc`] asks the run's pickers about
/// points a radius out from the centre.
///
/// **Which of the two circles is the side of the ring's own radius both
/// corners are on**, outside it for the outer circle and inside it for the
/// inner one. The two circles are `√(r² − h²)` either side of `R`, so a corner
/// on one is that far from `R`, on its own circle's side. That is the rule the reader names
/// the arc by when it reads it back (`which_of_a_rings_two_circles` in
/// `brep.rs`), so the circle this writes is the one the reader picks. Corners
/// on two sides, or exactly `R` from the axis, are refused.
///
/// A plane holding the axis crosses the ring in the tube's own two circles,
/// which a closed edge at the end of a bend is written as, by
/// [`where_a_cap_holding_a_rings_axis_crosses_it`]; and any other plane crosses
/// a ring in a curve with no name in this kernel.
fn where_a_plane_crosses_a_ring(
    flat: &Plane,
    ring: &Torus,
    corners: [Point; 2],
) -> Option<Crossing> {
    let along = ring.axis().direction();
    if flat.normal().cross(along) != Vector::zero() {
        return None;
    }
    let up = along * along.length_squared().recip();
    let reach = (flat.offset() - flat.normal().dot(ring.centre() - Point::origin()))
        / flat.normal().dot(up);
    let axial = reach * along.length();
    if !axial.is_finite() {
        return None;
    }
    let centre = ring.centre() + up * reach;
    let tube = ring.tube_radius();
    let half_width = (tube * tube - axial * axial).max(0.0).sqrt();
    let radius = if half_width == 0.0 {
        ring.ring_radius()
    } else {
        let outside_the_ring_radius = |corner: Point| {
            let out = corner - centre;
            let from_the_axis = (out - along * out.dot(up)).length();
            if from_the_axis > ring.ring_radius() {
                Some(true)
            } else if from_the_axis < ring.ring_radius() {
                Some(false)
            } else {
                None
            }
        };
        match corners.map(outside_the_ring_radius) {
            [Some(true), Some(true)] => ring.ring_radius() + half_width,
            [Some(false), Some(false)] => ring.ring_radius() - half_width,
            _ => return None,
        }
    };
    Some(Crossing {
        centre,
        turning: flat.normal(),
        shape: Shape::Circle { radius },
    })
}

/// Of the two circles a plane square to a ring's axis crosses it in, the one
/// nearer `near`, about the plane's normal; `None` where the plane is not
/// square to the axis or misses the tube.
///
/// A plane square to the axis `h` along it from the centre crosses the tube in
/// two circles about the axis, of radii `R ± √(r² − h²)`. Where it holds the
/// centre those are the inner and the outer equator, and the writer target's
/// bend has its seam along a quarter of the inner one. Which of the two an
/// edge is on is read off the edge's own corner, whose distance from the axis
/// is nearer one radius than the other by `√(r² − h²)`.
fn where_a_plane_square_to_a_ring_crosses_it_nearest(
    flat: &Plane,
    ring: &Torus,
    near: Point,
) -> Option<Crossing> {
    let along = ring.axis().direction();
    if flat.normal().cross(along) != Vector::zero() {
        return None;
    }
    let up = (flat.offset() - flat.normal().dot(ring.centre() - Point::origin()))
        / flat.normal().dot(along);
    let tube = ring.tube_radius();
    let spread = ((tube - up.abs()) * (tube + up.abs())).sqrt();
    if !spread.is_finite() {
        return None;
    }
    let centre = ring.centre() + along * up;
    let out = near - centre;
    let from_the_axis = (out - along * out.dot(along)).length();
    let [outer, inner] = [ring.ring_radius() + spread, ring.ring_radius() - spread];
    let radius = if (from_the_axis - outer).abs() <= (from_the_axis - inner).abs() {
        outer
    } else {
        inner
    };
    Some(Crossing {
        centre,
        turning: flat.normal(),
        shape: Shape::Circle { radius },
    })
}

/// The circle a flat cap crosses a cone in, worked out in the frame the two
/// faces are stored in and then placed; `None` where they are not stored in
/// one frame, the cap is not square to the cone's axis there, or it passes
/// through the apex.
///
/// **Square to the axis, and nothing else.** A plane square to a cone's axis
/// and off its apex crosses it in one circle, centred where the axis meets the
/// plane, of the cone's radius that far from the apex. Any other plane that
/// cuts a cone in a closed curve cuts it in an ellipse whose centre is off the
/// axis, and no builder or file this writer has met makes one, so it is not
/// built (`docs/decisions.md` §13) and refuses by name.
///
/// **Asked in the stored frame**, for [`Side`]'s reason: the frustum and the
/// pointed cone of the writer target are built square along `z`, and a turn
/// leaves the cap's normal and the cone's axis an ulp apart, which would read
/// as a plane that is not square at all (`docs/decisions.md` §366).
///
/// **It turns about the cone's axis**, as the pipe's circle turns about the
/// pipe's: that is the way `rcad-topo` stores a closed edge round a cone, since
/// [`Surface::the_way_its_first_coordinate_runs_at`] on a cone is
/// `a × (p − apex)`, a right hand's turn about `a` on either nappe, and the
/// reader reads each use of the edge against it
/// (`Walk::which_way_round_a_closed_edge_is_stored`).
fn where_a_cap_square_to_a_cone_crosses_it(sides: [&Side; 2]) -> Option<Crossing> {
    let [first, second] = sides;
    if first.placed_by != second.placed_by {
        return None;
    }
    let ((Surface::Plane(flat), Surface::Cone(cone)) | (Surface::Cone(cone), Surface::Plane(flat))) =
        (first.stored, second.stored)
    else {
        return None;
    };
    let along = cone.axis().direction();
    if flat.normal().cross(along) != Vector::zero() {
        return None;
    }
    let reached = (flat.offset() - flat.normal().dot(cone.apex() - Point::origin()))
        / flat.normal().dot(along);
    if reached == 0.0 || !reached.is_finite() {
        return None;
    }
    let placed_by = first.placed_by;
    Some(Crossing {
        centre: placed_by.apply_to_point(cone.apex() + along * reached),
        turning: placed_by.apply_to_vector(along),
        shape: Shape::Circle {
            radius: cone.radius_at(reached * along.length()),
        },
    })
}

/// The circle a flat cap holding a ring's axis crosses it in, on the side of
/// the axis `corner` is on, worked out in the frame the two faces are stored in
/// and then placed; `None` where they are not stored in one frame or the cap
/// does not hold the axis there.
///
/// **A plane holding the ring's axis crosses it in two circles round the
/// tube**, each of the tube's radius and centred the ring's radius out from the
/// centre, one either side of the axis. That is the end of a bend of pipe. A
/// plane along the axis but off it crosses a ring in a curve with no name in
/// this kernel, and so does any other plane but one square to the axis, so this
/// asks for exactly zero both ways: the normal square to the axis, and the
/// centre on the plane. Asked in the stored frame, for [`Side`]'s reason: the
/// writer target's bend is built about `z` with its caps in `y = 0` and
/// `x = 0`, and a turn would leave the cap's normal an ulp off square.
///
/// **Which of the two circles is the side of the axis the edge's one corner is
/// on**, read as a sign that is a ring's radius from zero.
///
/// **It turns the way the ring winds round its tube at the corner**, which is
/// how `rcad-topo` stores a closed edge on a ring and how the reader reads each
/// use of it back (`Walk::which_way_round_a_closed_edge_is_stored`).
/// [`Surface::the_way_its_first_coordinate_runs_at`] on a ring is the tangent
/// to its middle circle, `(p − c) × a`, crossed into the way the surface faces.
/// With `s = a × n` along the plane and square to the axis, a corner
/// `p − c = σ·|p − c|·s + h·a`, with `σ` the side, gives `(p − c) × a` along
/// `σ·n`, since `s × a = n`. So the circle on the side along `s` turns about the
/// cap's normal `n`, and the one on the other side about `−n`. That is the same
/// sign that picks the circle, and it needs no
/// `rcad_geom::which_way_a_closed_crossing_turns`.
fn where_a_cap_holding_a_rings_axis_crosses_it(
    sides: [&Side; 2],
    corner: Point,
) -> Option<Crossing> {
    let [first, second] = sides;
    if first.placed_by != second.placed_by {
        return None;
    }
    let ((Surface::Plane(flat), Surface::Torus(ring))
    | (Surface::Torus(ring), Surface::Plane(flat))) = (first.stored, second.stored)
    else {
        return None;
    };
    let along = ring.axis().direction();
    if flat.normal().dot(along) != 0.0 || flat.signed_distance_to(ring.centre()) != 0.0 {
        return None;
    }
    let out_from_the_axis = along.cross(flat.normal());
    let placed_by = first.placed_by;
    let side = (corner - placed_by.apply_to_point(ring.centre()))
        .dot(placed_by.apply_to_vector(out_from_the_axis));
    let (out_from_the_axis, turning) = if side > 0.0 {
        (out_from_the_axis, flat.normal())
    } else if side < 0.0 {
        (-out_from_the_axis, -flat.normal())
    } else {
        return None;
    };
    let middle =
        ring.centre() + out_from_the_axis * (ring.ring_radius() / out_from_the_axis.length());
    Some(Crossing {
        centre: placed_by.apply_to_point(middle),
        turning: placed_by.apply_to_vector(turning),
        shape: Shape::Circle {
            radius: ring.tube_radius(),
        },
    })
}

/// A point near `near` from which the reader's [`plane_through`] gives back
/// `cut`, normal and offset bit for bit, and which `held` takes, or `None`
/// where none of the few tried does.
///
/// **`held` is what the faces beside the edge ask of the circle's centre**
/// ([`a_centre_for_an_arc_between`]); a point that reads back as `cut` but
/// is not held is stepped past, as if it had missed. Every other caller
/// takes every point (`|_| true`).
///
/// `near` is where a pipe's axis or a ball's centre meets `cut`, worked out in
/// `f64`, so it can miss the offset by a rounding or two. The reader dots the
/// point with the normal and rounds once, which rises with any one coordinate
/// whose component of the normal is positive and falls with one whose
/// component is negative, so one coordinate is stepped an `f64` at a time
/// towards the offset until the dot lands on it or steps past it.
///
/// **The coordinate that moves the dot least per step is tried first**, then
/// the others: a step of one coordinate can move the dot by more than an ulp
/// of the offset and jump straight over it — `5.85` along a normal of
/// `−1/√2` does, on the two pipes of `Relay_SPDT_SANYOU_SRD_Series_Form_C`
/// — where a step of the other lands on it. A normal that does not come back
/// as itself is not helped by any point, and refuses at once.
///
/// **One coordinate is stepped at most fifteen `f64`s**: the point is read
/// before each step, sixteen times. A step is at most `ε` of the largest
/// number, the most where the coordinate stands on a power of two and steps
/// away from zero, so the centre moves at most `15ε` of it, and the reader
/// allows a circle `16ε` off its face (`A_CIRCLE_MAY_MISS_ITS_FACE_BY` in
/// `brep.rs`). That leaves one for the roundings of `near` and of the reader's
/// own bound, and nothing here argues one is enough. It is measured, with
/// nothing to spare: a ball one `f64` under `32` in radius at `(3, −32, 0)`,
/// turned `1.5709511621` about `z` and cut a sixteenth of its radius off its
/// centre, is stepped all fifteen along `y` and the reader's bound reads
/// `15.9998` (Q1789,
/// `a_ball_cut_near_its_middle_is_stepped_fifteen_times_to_the_edge_of_what_the_reader_holds`).
/// **That measurement is of a ball alone, and does not hold for a ball
/// beside a pipe**: a dome on a long bar, turned, was stepped fifteen along
/// `y` to `16.31` units off its ball, which is why an arc between two faces
/// passes a `held` that asks them (Q1792,
/// `a_dome_on_a_long_bar_turned_a_long_way_comes_back_with_its_rim_cut_from_its_plane`).
///
/// **Then the two finest coordinates are stepped together**, nearest first
/// and at most [`MOST_STEPS_OF_TWO_COORDINATES`] each way. Where the offset is
/// much smaller than the coordinates — a ball's seam turned to pass near the
/// origin — one step of any one coordinate moves the dot by several ulps of the
/// offset, and only a step of each lands between them: the turned ball of
/// radius three at `(10, 7, −15)` that `how_often_a_model_the_mutator_draws_is_refused`
/// draws has an offset of `0.80`, and a step of either coordinate moves the dot
/// six ulps of it or more (Q822).
fn a_centre_its_plane_reads_back_from(
    cut: &Plane,
    near: Point,
    held: impl Fn(Point) -> bool,
) -> Option<Point> {
    let normal = cut.normal();
    let read = |at: [f64; 3]| {
        plane_through(Point::new(at[0], at[1], at[2]), normal)
            .ok()
            .filter(|back| back.normal() == normal)
            .map(|back| back.offset())
    };
    let components = normal.components();
    let start = near.coordinates();
    let step_of = |axis: usize| components[axis].abs() * (start[axis].next_up() - start[axis]);
    let mut finest_first: Vec<usize> = (0..3).filter(|&axis| components[axis] != 0.0).collect();
    finest_first.sort_by(|&a, &b| step_of(a).total_cmp(&step_of(b)));
    let below = read(start)? < cut.offset();
    for &axis in &finest_first {
        let mut at = start;
        for _ in 0..16 {
            let offset = read(at)?;
            if offset.to_bits() == cut.offset().to_bits() {
                let centre = Point::new(at[0], at[1], at[2]);
                if held(centre) {
                    return Some(centre);
                }
            } else if (offset < cut.offset()) != below {
                break;
            }
            at[axis] = if below == (components[axis] > 0.0) {
                at[axis].next_up()
            } else {
                at[axis].next_down()
            };
        }
    }
    let [first, second, ..] = finest_first[..] else {
        return None;
    };
    let reads_back =
        |at: [f64; 3]| read(at).is_some_and(|offset| offset.to_bits() == cut.offset().to_bits());
    for ring in 1..=MOST_STEPS_OF_TWO_COORDINATES {
        for along_first in -ring..=ring {
            for along_second in -ring..=ring {
                if along_first.abs().max(along_second.abs()) != ring {
                    continue;
                }
                let mut at = start;
                at[first] = stepped(start[first], along_first);
                at[second] = stepped(start[second], along_second);
                let centre = Point::new(at[0], at[1], at[2]);
                if reads_back(at) && held(centre) {
                    return Some(centre);
                }
            }
        }
    }
    None
}

/// How many `f64`s either way [`a_centre_its_plane_reads_back_from`] steps
/// each of two coordinates together.
///
/// **The reader has to find the circle on the ball still**, and it allows a
/// circle `16·ε` of the largest number off (`A_CIRCLE_MAY_MISS_ITS_FACE_BY`
/// in `brep.rs`). One step of a coordinate is at most `ε` of it, the most
/// where it stands on a power of two and steps away from zero, so four steps
/// are at most `4ε` of it, and two coordinates stepped four each move the
/// centre at most `√2·4ε ≈ 5.66ε` of the larger. Every point of the circle
/// moves with its centre, so the circle moves off the ball by no more than
/// that, and the reader's sixteen leave ten for its roundings. The seam of a
/// ball of radius one at `(2, 2 + one ulp, 0)` turned `0.8178` about `z` is
/// stepped the full four up both coordinates from `2`, and the reader's bound
/// reads `5.75` (Q838,
/// `a_ball_seam_stepped_four_each_way_from_a_power_of_two_is_still_read_back_as_its_plane`).
const MOST_STEPS_OF_TWO_COORDINATES: i32 = 4;

/// `from` moved `steps` `f64`s up, or down for a negative count.
fn stepped(from: f64, steps: i32) -> f64 {
    let step = if steps > 0 {
        f64::next_up
    } else {
        f64::next_down
    };
    (0..steps.unsigned_abs()).fold(from, |at, _| step(at))
}

/// The centre nearest `crossing`'s own from which the reader's
/// [`plane_through`] gives back `cut` bit for bit and the reader still holds
/// the circle on every face of `either_side`, with two coordinates moved;
/// `None` where this finds none, or where a face has no bound the reader
/// could hold the circle to.
///
/// **Where the offset is much smaller than the coordinates, a step of one
/// coordinate moves the dot by dozens of ulps of it**, so the dot lands on the
/// offset only at a few points of the lattice, which can lie twenty steps
/// out, past the four [`a_centre_its_plane_reads_back_from`] tries. The writer
/// target's bend turned about `y` by `5.0` and moved by `(3, 5, 10)` has its
/// seam's offset `0.04`, the turned normal dotted with the move in plain `f64`
/// and `32` ulps from the ring's centre dotted and rounded once. A step of `x`
/// moves the dot `61` ulps and a step of `z` `73`, and the one centre the
/// reader holds is `23` steps of `x` and `19` of `z` out, at `15.88` of the
/// sixteen units it allows (Q843,
/// `a_turned_bends_seam_has_one_centre_its_plane_reads_back_from`).
///
/// **So one coordinate is walked an `f64` at a time, and for each step the
/// other is put where the dot comes back**, then the `f64`s either side of
/// that are tried too, since the dot is not worked out exactly here and
/// [`plane_through`] is what decides. Every pair of the coordinates the normal
/// leans along is walked both ways, out to twice the reader's allowance for
/// the face whose bound is largest, or [`MOST_STEPS_OF_ONE_COORDINATE`],
/// whichever comes first.
///
/// **What the reader holds is asked here, not trusted to a bound on the
/// steps** ([`a_surface_holds_the_circle`] in `brep.rs`, shared with the
/// reader rather than copied, since Q957). Between two faces, a centre that
/// reads back as the plane but sits further off a face than the reader
/// allows has the arc named by its faces instead: a turned dome on a bar,
/// centred where the walk lands ten `f64`s of `x` out, came back with its
/// rim cut from the ball, and this finds the centre ten of `x` and sixteen
/// of `z` out that both faces hold (Q1792,
/// `a_dome_on_a_long_bar_turned_a_little_comes_back_with_its_rim_cut_from_its_plane`).
/// A ring's seam has one face on both sides, which the reader does not ask
/// ([`a_centre_for_an_arc_between`]); it is asked here all the same, so the
/// circle the file carries lies on the face it bounds.
fn the_nearest_centre_its_faces_hold_the_circle_at(
    cut: &Plane,
    either_side: &[&Surface],
    crossing: &Crossing,
) -> Option<Point> {
    let Shape::Circle { radius } = crossing.shape else {
        return None;
    };
    let normal = cut.normal();
    let components = normal.components();
    let start = crossing.centre.coordinates();
    let short = cut.offset() - plane_through(crossing.centre, normal).ok()?.offset();
    let largest = either_side.iter().try_fold(0.0_f64, |most, face| {
        how_far_off_the_circle_can_be(face, crossing.centre, normal, radius)
            .map(|(_, largest)| most.max(largest))
    })?;
    let reach = 2.0 * A_CIRCLE_MAY_MISS_ITS_FACE_BY * NEAREST_F64_POINT_ROUNDOFF * largest;
    let leaning: Vec<usize> = (0..3).filter(|&axis| components[axis] != 0.0).collect();
    let mut nearest: Option<(f64, Point)> = None;
    for &walked in &leaning {
        for &solved in leaning.iter().filter(|&&solved| solved != walked) {
            for step in [f64::next_up as fn(f64) -> f64, f64::next_down] {
                let mut at = start;
                for _ in 0..=MOST_STEPS_OF_ONE_COORDINATE {
                    let moved = at[walked] - start[walked];
                    if moved.abs() > reach {
                        break;
                    }
                    let landing =
                        start[solved] + (short - components[walked] * moved) / components[solved];
                    for tried in [landing.next_down(), landing, landing.next_up()] {
                        let mut centre = at;
                        centre[solved] = tried;
                        let centre = Point::new(centre[0], centre[1], centre[2]);
                        let reads_back = plane_through(centre, normal).is_ok_and(|back| {
                            back.normal() == normal
                                && back.offset().to_bits() == cut.offset().to_bits()
                        });
                        if !reads_back
                            || !either_side.iter().all(|face| {
                                a_surface_holds_the_circle(face, centre, normal, radius)
                            })
                        {
                            continue;
                        }
                        let off = (centre - crossing.centre).length();
                        if nearest.is_none_or(|(nearest_off, _)| off < nearest_off) {
                            nearest = Some((off, centre));
                        }
                    }
                    at[walked] = step(at[walked]);
                }
            }
        }
    }
    nearest.map(|(_, centre)| centre)
}

/// How many `f64`s either way [`the_nearest_centre_its_faces_hold_the_circle_at`]
/// walks one coordinate at most.
///
/// An `f64` is at least `ε/2` of itself, so sixty-four steps of the largest
/// coordinate reach `32·ε` of it, which is twice what the reader allows a
/// circle to miss its face by. A smaller coordinate stops short of that, and
/// the larger one, walked in its turn, covers the rest.
const MOST_STEPS_OF_ONE_COORDINATE: usize = 64;

impl Crossing {
    /// Which way round this curve to write an arc of it, so that the arc from
    /// `start` to `end` is the one `run` picks; `None` where the run's
    /// pickers do not tell the two arcs apart.
    ///
    /// **The reader's rule, run backwards.** A `CIRCLE` or an `ELLIPSE` runs
    /// the way a right hand turns about its axis `w`, and the reader names the
    /// arc from `A` to `B` by the half-space facing `(B − A) × w`, which every
    /// point of that arc is strictly in front of (`Walk::the_arc_between`).
    /// So `w` is this curve's own way round where the arc the run picks is in
    /// front of `(B − A) × w`, and the other way where it is behind.
    ///
    /// **Which arc the run picks is asked of its own pickers**, at the one
    /// point of each arc furthest from the chord: the curve's furthest point
    /// either way along `(B − A) × w`. One of the two has to be held by every
    /// picker and the other ruled out by one, or this refuses. That is how
    /// `rcad-topo` reads a picked arc too (`EdgeView::length`), and it never
    /// picks the wrong arc on rounding alone: a picker that tells the arcs
    /// apart rules the other arc's furthest point out, so the wrong answer
    /// takes both points on the wrong side at once — the picked arc's ruled
    /// out by some picker *and* the other's held by every one — and a single
    /// probe that rounding moves across a picker leaves both held or both
    /// ruled out, which refuses.
    fn the_way_round_the_arc(&self, run: &EdgeRun, start: Point, end: Point) -> Option<Vector> {
        let its_own_way = self.the_arc_runs_its_own_way(run, start, end)?;
        Some(if its_own_way {
            self.turning
        } else {
            -self.turning
        })
    }

    /// Whether the arc from `start` to `end` that `run` picks is the one this
    /// curve's own way round takes; `None` where the run's pickers do not tell
    /// the two arcs apart. See [`Crossing::the_way_round_the_arc`].
    fn the_arc_runs_its_own_way(&self, run: &EdgeRun, start: Point, end: Point) -> Option<bool> {
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run else {
            return None;
        };
        let into_the_arc = (end - start).cross(self.turning);
        let furthest = self.furthest_either_way_along(into_the_arc)?;
        match furthest.map(|point| what_the_pickers_make_of(picked_by, point)) {
            [Verdict::Holds, Verdict::RuledOut] => Some(true),
            [Verdict::RuledOut, Verdict::Holds] => Some(false),
            _ => None,
        }
    }

    /// Whether this curve, about its own way round, turns at `corner` the way
    /// a closed edge between `either_side` is stored running; `None` where
    /// the two are square there or the curved face has no way round at it.
    ///
    /// **A closed edge is stored running the way the curved face beside it
    /// winds at its one corner**, and of two curved faces the one
    /// [`Surface::how_it_sorts_against`] puts first, which is a ball before a
    /// pipe: `rcad-topo`'s and the reader's `the_curved_one_of`. The reader
    /// reads which way a use of the edge goes by dotting the file's curve's
    /// turn, `w × (corner − centre)`, with that winding in plain `f64`
    /// (`Walk::which_way_round_a_closed_edge_is_stored`), so this asks the
    /// same dot of the curve it is about to write. A plane through a ball's
    /// poles crosses it in a circle square to its winding everywhere, and is
    /// refused here as the model's own rule refuses it.
    ///
    /// [`where_a_plane_crosses_a_ball`] turns its circle about the plane's
    /// normal, and a flat face under a ball faces away from the way the ball
    /// winds: written about that normal, the cap
    /// `a_cap_cut_off_a_ball_along_one_closed_circle_is_written_and_comes_back_the_same`
    /// draws was refused by the reader as "not a solid this kernel can hold".
    fn it_runs_the_way_a_closed_edge_is_stored(
        &self,
        either_side: [&Surface; 2],
        corner: Point,
    ) -> Option<bool> {
        let curved = the_curved_one_of(&either_side.map(|face| *face))?;
        let winding = curved.the_way_its_first_coordinate_runs_at(corner)?;
        let leaning = self.turning.cross(corner - self.centre).dot(winding);
        if leaning > 0.0 {
            Some(true)
        } else if leaning < 0.0 {
            Some(false)
        } else {
            None
        }
    }

    /// The point of this curve furthest along `way`, and the one furthest
    /// against it; `way` lies in the curve's plane.
    ///
    /// On an ellipse `c + p·cos t·e₁ + q·sin t·e₂` the distance along `way` is
    /// `P·cos t + Q·sin t` with `P = p·(e₁·way)` and `Q = q·(e₂·way)`, which is
    /// furthest where `(cos t, sin t)` is `(P, Q)` scaled to length one. A
    /// circle is the ellipse whose two sizes are one, which puts that point a
    /// radius along `way` itself.
    fn furthest_either_way_along(&self, way: Vector) -> Option<[Point; 2]> {
        let reach = match self.shape {
            Shape::Circle { radius } => way.normalized().ok()? * radius,
            Shape::Ellipse { first_way, sizes } => {
                let first_way = first_way.normalized().ok()?;
                let second_way = self.turning.normalized().ok()?.cross(first_way);
                let [p, q] = [
                    sizes[0] * first_way.dot(way),
                    sizes[1] * second_way.dot(way),
                ];
                let length = p.hypot(q);
                if !(length > 0.0 && length.is_finite()) {
                    return None;
                }
                first_way * (sizes[0] * p / length) + second_way * (sizes[1] * q / length)
            }
        };
        Some([self.centre + reach, self.centre - reach])
    }
}

/// What the pickers of a run make of one point.
enum Verdict {
    /// Every picker holds it, on its boundary or inside.
    Holds,
    /// Some picker rules it out.
    RuledOut,
    /// No picker rules it out, and some picker cannot tell.
    CannotTell,
}

fn what_the_pickers_make_of(pickers: &[Picker], point: Point) -> Verdict {
    let at = PredicatePoint::from(point.coordinates());
    let mut verdict = Verdict::Holds;
    for picker in pickers {
        match picker.side_of_point(at) {
            Decision::Pos | Decision::Zero => {}
            Decision::Neg => return Verdict::RuledOut,
            // `Decision` grows (docs/decisions.md §40), and a sign this has
            // never seen tells it nothing about which arc this is.
            Decision::Undecidable { .. } | _ => verdict = Verdict::CannotTell,
        }
    }
    verdict
}

/// How many `f64` steps either side of `atan(slope)` [`a_half_angle_that_reads_back_as`]
/// looks for an angle whose tangent is the slope.
const HALF_ANGLE_STEPS_SEARCHED: usize = 64;

/// The half-angle to write for a cone of `slope`: the `f64` nearest
/// `atan(slope)` whose tangent is `slope` bit for bit, since the reader takes
/// the tangent of what it reads (`cone_from`), or, where none within
/// [`HALF_ANGLE_STEPS_SEARCHED`] steps has one, the angle among those whose
/// tangent comes nearest.
///
/// **`atan` alone misses slopes the reader can be given exactly.** `atan(0.5)`
/// reads back as `0.49999999999999994` and the `f64` above it as `0.5`, which
/// is one of the four slopes the writer target draws
/// (`each_slope_the_writer_target_draws_reads_back_from_the_angle_written`).
/// **For most slopes over one no angle reads back at all**: a step of the
/// angle moves the tangent by about `1 + s²` times as much, which for `s` over
/// one skips slopes, so that slope comes back as near as an angle can carry it
/// and no nearer. That is a limit and not a gap; `docs/decisions.md` §366 has
/// the count and the demonstration.
fn a_half_angle_that_reads_back_as(slope: f64) -> f64 {
    let miss = |angle: f64| (angle.tan() - slope).abs();
    let mut nearest = slope.atan();
    let mut nearest_miss = miss(nearest);
    let (mut above, mut below) = (nearest, nearest);
    for _ in 0..HALF_ANGLE_STEPS_SEARCHED {
        if nearest_miss == 0.0 {
            break;
        }
        above = above.next_up();
        below = below.next_down();
        for angle in [above, below] {
            let missed_by = miss(angle);
            if missed_by < nearest_miss {
                nearest = angle;
                nearest_miss = missed_by;
            }
        }
    }
    nearest
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
/// The reader keeps a normal that is only within
/// `Plane::NORMAL_LENGTH_ROUNDOFF` of length one, and one of those can have
/// its widest component within `32ε` of one and the other two zero, with no
/// `nⱼ` to divide by. There the one coordinate is stepped an `f64` at a time
/// until it reads back ([`stepped_until_it_reads_back`]).
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
    if left_over == 0.0 {
        return coordinates;
    }
    if normal[next] == 0.0 {
        coordinates[widest] =
            stepped_until_it_reads_back(offset, normal[widest], coordinates[widest]);
    } else {
        coordinates[next] = left_over / normal[next];
    }
    coordinates
}

/// The coordinate along a normal's only component that is not zero from which
/// the reader works out `offset`, found by stepping one `f64` at a time from
/// `start`, the quotient `offset / along`.
///
/// **Only a normal the reader keeps reaches this.** With its other two
/// components zero it has no second coordinate to carry what the first left
/// over, and its one component is not ±1 — `(0, 0, −1.0000000000000002)` is
/// one a turn left — so `offset / along` can land an ulp or two from a
/// coordinate that reads back. The reader's dot is then `along · z` rounded
/// once, which rises with `z` when `along` is positive and falls when it is
/// not, so the walk goes one way and stops at the first coordinate that reads
/// back as `offset` or past it. A step moves the product by about an ulp of
/// the offset, and the quotient starts within one, so eight steps is far more
/// than it takes; one that finds nothing hands back a coordinate the check in
/// [`a_point_this_reader_reads_back_as`] refuses by name.
fn stepped_until_it_reads_back(offset: f64, along: f64, start: f64) -> f64 {
    let mut coordinate = start;
    let below = along * coordinate < offset;
    for _ in 0..8 {
        let read = along * coordinate;
        if read == offset || (read < offset) != below {
            return coordinate;
        }
        coordinate = if below == (along > 0.0) {
            coordinate.next_up()
        } else {
            coordinate.next_down()
        };
    }
    coordinate
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
    use core::f64::consts::{FRAC_PI_2, SQRT_2, TAU};

    use rcad_geom::{
        Axis, Cylinder, EdgeRun, HalfSpace, Isometry, NEAREST_F64_POINT_ROUNDOFF, Orientation,
        Picker, Plane, Point, Scale, Sphere, Surface, Torus, Vector,
    };
    use rcad_predicates::{Policy, ProvenanceLog};
    use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey, WireView};

    use super::{
        A_CIRCLE_MAY_MISS_ITS_FACE_BY, CannotWrite, Shape, a_centre_its_plane_reads_back_from,
        a_point_this_reader_reads_back_as, a_surface_holds_the_circle,
        how_far_off_the_circle_can_be, stepped, the_nearest_centre_its_faces_hold_the_circle_at,
        triple, where_a_plane_crosses_a_ball, where_a_plane_crosses_a_pipe,
        where_a_plane_square_to_a_ring_crosses_it_nearest, write,
    };
    use crate::geometry::plane_through;
    use crate::{Import, StepFile};

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

    /// The seam of the writer target's quarter bend (ring seven, tube one),
    /// turned about `y` by `5.0` and moved by `(3, 5, 10)`: one representable
    /// centre reads back as its plane and lies on the ring as the reader asks
    /// it, and the writer finds it. Four steps each way of two coordinates, all
    /// [`a_centre_its_plane_reads_back_from`] tries, do not reach it, and the
    /// writer refused the seam until
    /// [`the_nearest_centre_its_faces_hold_the_circle_at`] (Q843).
    ///
    /// **The plane is the seam's own, not a rounding off square.** Its normal
    /// and the ring's axis are the same turn of `z`, bit for bit, so the two
    /// cross to exactly nothing. Its offset is the turned normal dotted with
    /// `(3, 5, 10)` in plain `f64` (`Plane::transformed`), which is `32` ulps
    /// from the reader's dot of the ring's centre, rounded once.
    ///
    /// **Every centre the reader could hold is searched.** A file can carry
    /// any `f64` point as a `CIRCLE`'s centre, and the reader holds a circle
    /// about a ring's axis when its centre is within `16·ε` of the largest
    /// number of the axis (`how_far_off_the_circle_can_be` in `brep.rs`). The
    /// normal has no `y`, so moving `y` leaves the dot alone and only moves
    /// the centre off the axis; every centre the reader could hold then has
    /// `x` and `z` within twice that of the ring's centre, and those are all
    /// tried. Of them exactly one reads back as the plane and is held: `23`
    /// steps down in `x` and `19` in `z`, at `15.88` of the sixteen units the
    /// reader allows. So this is a gap and not a limit (`docs/decisions.md`
    /// §16.10): the answer exists, and four steps each way of two coordinates
    /// was where the search stopped. The reader's rule is asked through the
    /// writer's copy of it; that the reader itself holds this centre is
    /// `how_often_a_turned_bend_comes_back_from_its_own_file`, which reads the
    /// bend back.
    #[test]
    fn a_turned_bends_seam_has_one_centre_its_plane_reads_back_from() {
        let turned = Isometry::translation(Vector::new(3.0, 5.0, 10.0))
            .and_then(|moved| Isometry::rotation(&Axis::y(), 5.0).map(|turn| moved.compose(&turn)))
            .expect("a finite move and turn");
        let through_the_middle =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal");
        let Surface::Plane(cut) = Surface::Plane(through_the_middle).transformed(&turned) else {
            unreachable!("a plane moves as a plane");
        };
        let Surface::Torus(ring) =
            Surface::Torus(Torus::new(Axis::z(), 7.0, 1.0).expect("a tube narrower than its ring"))
                .transformed(&turned)
        else {
            unreachable!("a ring moves as a ring");
        };
        let normal = cut.normal();
        assert_eq!(normal.cross(ring.axis().direction()), Vector::zero());
        let centre = ring.centre();
        let read_back = plane_through(centre, normal)
            .expect("a finite point")
            .offset();
        assert_eq!(read_back.to_bits() - cut.offset().to_bits(), 32);

        let corner = turned.apply_to_point(Point::new(6.0, 0.0, 0.0));
        let crossing = where_a_plane_square_to_a_ring_crosses_it_nearest(&cut, &ring, corner)
            .expect("a plane square to the axis through the ring's centre");
        let Shape::Circle { radius } = crossing.shape else {
            unreachable!("a plane square to a ring's axis crosses it in circles");
        };
        assert_eq!((crossing.centre, radius), (centre, 6.0));

        let [x, y, z] = centre.coordinates();
        let reach = 2.0 * A_CIRCLE_MAY_MISS_ITS_FACE_BY * NEAREST_F64_POINT_ROUNDOFF * 10.0;
        let steps = |of: f64| (reach / (of.next_up() - of)).ceil() as i32;
        let mut found = Vec::new();
        for along_x in -steps(x)..=steps(x) {
            for along_z in -steps(z)..=steps(z) {
                let at = Point::new(stepped(x, along_x), y, stepped(z, along_z));
                let reads_back = plane_through(at, normal)
                    .is_ok_and(|back| back.offset().to_bits() == cut.offset().to_bits());
                if reads_back
                    && a_surface_holds_the_circle(&Surface::Torus(ring), at, normal, radius)
                {
                    found.push((along_x, along_z));
                }
            }
        }
        assert_eq!(found, [(-23, -19)]);
        assert_eq!(
            a_centre_its_plane_reads_back_from(&cut, centre, |_| true),
            None
        );
        assert_eq!(
            the_nearest_centre_its_faces_hold_the_circle_at(
                &cut,
                &[&Surface::Torus(ring)],
                &crossing
            ),
            Some(Point::new(stepped(x, -23), y, stepped(z, -19)))
        );
    }

    /// The seam of `add_ball(3)` turned `149°` about `z` and moved to
    /// `(10, −6, 5)`: no centre the reader holds on the ball reads back as the
    /// plane the seam is cut from, so no file carries that seam as it is, and
    /// the ball is written cut open where `add_ball` cuts it (Q1308).
    ///
    /// **The plane passes near the origin and the ball does not.** Its offset
    /// is `−7.4e-3`, the turned normal dotted with `(10, −6, 5)` in plain
    /// `f64` (`Plane::transformed`), and the reader's dot of the ball's centre,
    /// rounded once, is 256 ulps of it away. One `f64` step of `x` or `y` moves
    /// that dot by `|n_x|·ulp(10)` or `|n_y|·ulp(6)`, about `9.2e-16` and
    /// `7.6e-16`, which is about a thousand ulps of the offset each.
    ///
    /// **Every centre the reader could hold is searched**, as for the bend
    /// above. The normal has no `z`, so moving `z` leaves the dot alone. A
    /// centre the reader holds is within `16·ε` of `10` of the ball's centre
    /// across the normal (`how_far_off_the_circle_can_be` in `brep.rs`), and
    /// one whose plane reads back is within a few ulps of the offset of it
    /// along the normal, so its `x` and `y` are within twice that allowance of
    /// the ball's centre; those are all tried. 7 385 are held, and not one
    /// reads back. So for this ball the refusal was a limit of a seam carried
    /// by its circle's centre, not a gap a wider search would close
    /// (`docs/decisions.md` §16.10).
    #[test]
    fn no_centre_the_reader_holds_on_the_ball_carries_the_seam_of_a_ball_turned_149_degrees() {
        let turned = Isometry::translation(Vector::new(10.0, -6.0, 5.0))
            .and_then(|moved| {
                Isometry::rotation(&Axis::z(), 149_f64.to_radians())
                    .map(|turn| moved.compose(&turn))
            })
            .expect("a finite move and turn");
        let add_balls_cut =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let Surface::Plane(cut) = Surface::Plane(add_balls_cut).transformed(&turned) else {
            unreachable!("a plane moves as a plane");
        };
        let Surface::Sphere(ball) =
            Surface::Sphere(Sphere::new(Point::origin(), 3.0).expect("a radius of three"))
                .transformed(&turned)
        else {
            unreachable!("a ball moves as a ball");
        };
        let normal = cut.normal();
        let [x, y, z] = ball.centre().coordinates();
        assert_eq!((x, y, z), (10.0, -6.0, 5.0));
        assert_eq!(normal.components()[2], 0.0);
        assert_eq!(cut.offset(), -7.376_944_887_870_884e-3);
        let read_back = plane_through(ball.centre(), normal)
            .expect("a finite point")
            .offset();
        assert_eq!(read_back.to_bits().abs_diff(cut.offset().to_bits()), 256);

        let reach = 2.0 * A_CIRCLE_MAY_MISS_ITS_FACE_BY * NEAREST_F64_POINT_ROUNDOFF * 10.0;
        let steps = |of: f64| (reach / (of.next_up() - of)).ceil() as i32;
        let skin = Surface::Sphere(ball);
        let (mut held, mut read_back_as_the_offset) = (0, 0);
        for along_x in -steps(x)..=steps(x) {
            for along_y in -steps(y)..=steps(y) {
                let at = Point::new(stepped(x, along_x), stepped(y, along_y), z);
                if !a_surface_holds_the_circle(&skin, at, normal, ball.radius()) {
                    continue;
                }
                held += 1;
                if plane_through(at, normal)
                    .is_ok_and(|back| back.offset().to_bits() == cut.offset().to_bits())
                {
                    read_back_as_the_offset += 1;
                }
            }
        }
        assert_eq!((held, read_back_as_the_offset), (7385, 0));
        assert_eq!(
            a_centre_its_plane_reads_back_from(&cut, ball.centre(), |_| true),
            None
        );
    }

    /// The seam of a ball of radius one at `(2, 2 + one ulp, 0)`, turned
    /// `0.8178` about `z`: the writer centres its circle four `f64`s up from
    /// the ball's centre in both `x` and `y`, the furthest
    /// [`MOST_STEPS_OF_TWO_COORDINATES`](super::MOST_STEPS_OF_TWO_COORDINATES)
    /// lets it go, and the reader still names the seam by its plane (Q838).
    ///
    /// **Only four up in both lands on the offset.** The plane is
    /// `−0.0917` from the origin, and the reader's dot of the ball's centre
    /// rounds to `6` ulps of it away. A step of `x` moves that dot about `23`
    /// ulps one way and a step of `y` about `22` the other, so no step of one
    /// coordinate lands on it, and a step of each moves it about `1.5`.
    ///
    /// **This is the most four steps of each can move a centre.** `x` stands
    /// on `2`, a power of two, and steps away from zero, so each step is `ε`
    /// of it, the largest a step of any coordinate is. The centre moves
    /// `√2·4ε` of `2`, and the reader's bound for the circle on the ball
    /// (`how_far_off_the_circle_can_be` in `brep.rs`) reads `5.75` of the
    /// sixteen units it allows.
    #[test]
    fn a_ball_seam_stepped_four_each_way_from_a_power_of_two_is_still_read_back_as_its_plane() {
        let turned = Isometry::translation(Vector::new(2.0, 2.0_f64.next_up(), 0.0))
            .and_then(|moved| {
                Isometry::rotation(&Axis::z(), 0.817_814_089_933_346_3)
                    .map(|turn| moved.compose(&turn))
            })
            .expect("a finite move and turn");
        let add_balls_cut =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let Surface::Plane(cut) = Surface::Plane(add_balls_cut).transformed(&turned) else {
            unreachable!("a plane moves as a plane");
        };
        let Surface::Sphere(ball) =
            Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a radius of one"))
                .transformed(&turned)
        else {
            unreachable!("a ball moves as a ball");
        };
        let normal = cut.normal();
        let [x, y, z] = ball.centre().coordinates();
        assert_eq!(cut.offset(), -0.091_670_029_559_456_13);
        let read_back = plane_through(ball.centre(), normal)
            .expect("a finite point")
            .offset();
        assert_eq!(read_back.to_bits().abs_diff(cut.offset().to_bits()), 6);
        let ulp_of_the_offset = cut.offset().abs().next_up() - cut.offset().abs();
        let [along_x, along_y, _] = normal
            .components()
            .map(|component| component / ulp_of_the_offset);
        let dot_moved_by_a_step = [along_x * (x.next_up() - x), along_y * (y.next_up() - y)];
        assert_eq!(dot_moved_by_a_step.map(f64::round), [-23.0, 22.0]);

        let centre = a_centre_its_plane_reads_back_from(&cut, ball.centre(), |_| true)
            .expect("a centre is found");
        assert_eq!(centre, Point::new(stepped(x, 4), stepped(y, 4), z));
        assert_eq!(x, 2.0);
        assert_eq!(x.next_up() - x, NEAREST_F64_POINT_ROUNDOFF * x);
        let (off, largest) =
            how_far_off_the_circle_can_be(&Surface::Sphere(ball), centre, normal, ball.radius())
                .expect("a ball has a bound");
        let moved = (centre - ball.centre()).length() / (NEAREST_F64_POINT_ROUNDOFF * largest);
        let units = off / (NEAREST_F64_POINT_ROUNDOFF * largest);
        println!("the centre moved {moved} roundings, and the reader's bound reads {units}");
        assert!((moved - 4.0 * SQRT_2).abs() < 1e-12, "{moved}");
        assert!((units - 5.75).abs() < 0.01, "{units}");
        assert!(a_surface_holds_the_circle(
            &Surface::Sphere(ball),
            centre,
            normal,
            ball.radius()
        ));

        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let solid = model
            .add_ball(1.0, &Isometry::identity())
            .expect("a ball of radius one fits");
        model
            .place_solid(solid, &turned)
            .expect("the key came from this model");
        let went_in = the_plane_its_seam_is_cut_from(&model);
        assert_eq!(format!("{went_in:?}"), format!("{cut:?}"));
        let text = write(&model, "a ball turned near a power of two").expect("the ball is written");
        assert!(text.contains(&triple(centre.coordinates())), "{text}");
        let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
        let back = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a ball this writer wrote is read back");
        assert_eq!(
            format!("{:?}", the_plane_its_seam_is_cut_from(back.model())),
            format!("{went_in:?}")
        );
    }

    /// A ball one `f64` under `32` in radius at `(3, −32, 0)`, turned
    /// `1.5709511621` about `z` and cut a sixteenth of its radius off its
    /// centre: the writer centres the circle the plane cuts it in fifteen
    /// `f64`s along `y` from where the plane meets the line through the
    /// centre, and the reader's bound for the circle on the ball reads
    /// `15.9998` of the sixteen units it allows (Q1789).
    ///
    /// **Fifteen is the most one coordinate is stepped**, and each step is at
    /// most `ε` of the largest number: `y` is just past `−32` and steps away
    /// from zero, so each step is `ε` of `32`. The turn leaves the normal
    /// `1.55e-4` off `x`, so a step of `y` moves the dot a small part of an ulp
    /// of the offset, and `y` is tried first.
    ///
    /// **The rest of the sixteen is roundings**: the circle's centre and
    /// radius as the crossing works them out, and the reader's own. A radius
    /// one `f64` under a power of two puts the reader's root on the finer side
    /// of it, where it rounds in half units. A probe that swept some twelve
    /// million balls placed and sized near powers of two and turned near a
    /// quarter turn about `z` found none past sixteen and the furthest at
    /// `15.9999993`, with a radius and a place one `f64` off; this one, with
    /// whole numbers for its place, is the furthest of those it tried with
    /// that radius. So sixteen holds this walk by measurement, with nothing to
    /// spare, and not by an argument.
    #[test]
    fn a_ball_cut_near_its_middle_is_stepped_fifteen_times_to_the_edge_of_what_the_reader_holds() {
        let radius = 32_f64.next_down();
        let turned = Isometry::translation(Vector::new(3.0, -32.0, 0.0))
            .and_then(|moved| {
                Isometry::rotation(&Axis::z(), 1.570_951_162_100_365_8)
                    .map(|turn| moved.compose(&turn))
            })
            .expect("a finite move and turn");
        let a_sixteenth_off =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .and_then(|plane| plane.moved_to(radius / 16.0))
                .expect("the y axis is a unit normal");
        let Surface::Plane(cut) = Surface::Plane(a_sixteenth_off).transformed(&turned) else {
            unreachable!("a plane moves as a plane");
        };
        let Surface::Sphere(ball) =
            Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a radius under 32"))
                .transformed(&turned)
        else {
            unreachable!("a ball moves as a ball");
        };
        let crossing = where_a_plane_crosses_a_ball(&cut, &ball).expect("the plane cuts the ball");
        let Shape::Circle { radius: across } = crossing.shape else {
            unreachable!("a plane crosses a ball in a circle");
        };
        let normal = cut.normal();
        let [along_x, along_y, _] = normal.components();
        assert!((along_x + 1.0).abs() < 1e-7 && (along_y + 1.55e-4).abs() < 1e-6);

        let centre = a_centre_its_plane_reads_back_from(&cut, crossing.centre, |_| true)
            .expect("a centre is found");
        let [x, y, z] = crossing.centre.coordinates();
        assert_eq!(centre, Point::new(x, stepped(y, -15), z));
        assert!(y < -32.0 && stepped(y, -15) > -64.0);
        assert_eq!(y - y.next_down(), NEAREST_F64_POINT_ROUNDOFF * 32.0);
        let skin = Surface::Sphere(ball);
        let (off, largest) = how_far_off_the_circle_can_be(&skin, centre, normal, across)
            .expect("a ball has a bound");
        let units = off / (NEAREST_F64_POINT_ROUNDOFF * largest);
        println!("the reader's bound reads {units}");
        assert!((units - 15.9998).abs() < 1e-4, "{units}");
        assert!(units <= A_CIRCLE_MAY_MISS_ITS_FACE_BY, "{units}");
        assert!(a_surface_holds_the_circle(&skin, centre, normal, across));
    }

    /// A bar of radius just over `16` and length `3561`, with a dome of radius
    /// `29.7` on top, moved to just off `(0, −2, 4096)` and turned
    /// `2.97` about a line mostly along `y`: the rim where the dome meets the
    /// bar comes back cut from its own plane (Q1792).
    ///
    /// **Walking `y` alone left the rim further off the dome than the reader
    /// allows.** The finest coordinate was `y`, and fifteen `f64`s of it put
    /// the dot on the plane's offset, with the circle `16.31` units off the
    /// ball where the reader allows sixteen. So the reader named the rim by
    /// the pipe and the ball instead, and the volume read back moved from
    /// `2865724.56` to `2865939.99`. Four `f64`s down in `x` land on the
    /// offset too, and there both faces hold the circle, the ball at `15.03`
    /// units.
    #[test]
    fn a_dome_on_a_long_bar_turned_a_long_way_comes_back_with_its_rim_cut_from_its_plane() {
        let axis = Axis::new(
            Point::origin(),
            Vector::new(
                -0.018_298_610_974_051_055,
                0.374_487_539_909_369_2,
                -0.017_226_945_673_374_46,
            ),
        )
        .expect("a direction with a length");
        the_rim_comes_back_cut_from_its_plane(
            (
                16.000_000_000_000_007,
                3561.383_252_900_813,
                29.695_524_847_710_807,
            ),
            Vector::new(0.0, -1.999_999_999_999_999_8, 4_095.999_999_999_999_5),
            &Isometry::rotation(&axis, 2.972_547_828_691_298_3).expect("a finite turn"),
            [-4, 0, 0],
        );
    }

    /// A bar of radius just under one and length `480`, with a dome of radius
    /// `1.84` on top, moved to just off `(512, −1, −4)` and turned by quarter
    /// turns and a small wobble: the rim comes back cut from its own plane
    /// (Q1792).
    ///
    /// **The walk landed ten `f64`s of `x` out**, with the circle `16.91` units
    /// off the ball, and the reader named the rim by the pipe and the ball
    /// instead. The nearest centre both faces hold is ten `f64`s up in `x` and
    /// sixteen down in `z`, past the four each way that the walk of two
    /// coordinates tries, and the ball holds the circle there at `15.96`
    /// units.
    #[test]
    fn a_dome_on_a_long_bar_turned_a_little_comes_back_with_its_rim_cut_from_its_plane() {
        the_rim_comes_back_cut_from_its_plane(
            (
                0.999_999_999_999_999_8,
                479.722_701_923_535_57,
                1.844_030_436_432_878_6,
            ),
            Vector::new(
                511.999_999_999_999_94,
                -0.999_999_999_999_999_8,
                -4.000_000_000_000_002,
            ),
            &a_small_turn_then_quarter_turns(
                Vector::new(
                    -0.229_389_415_830_124_18,
                    -0.324_935_301_137_225_35,
                    0.004_735_058_071_504_095,
                ),
                0.001_509_429_458_854_325,
                3.0,
                1.0,
            ),
            [10, 0, -16],
        );
    }

    /// A bar of radius `1/8` and length `1955`, with half a ball of the same
    /// radius on top, moved to just off `(2048, 0, −64)` and turned
    /// by quarter turns and a small wobble: it is read back, with the rim cut
    /// from its own plane (Q1792).
    ///
    /// **Our own reader refused it.** The walk landed four `f64`s of `x` out,
    /// with the circle `17.41` units off the ball, so the reader named the rim
    /// by the pipe and the ball; and a ball of the bar's own radius only
    /// touches the pipe there, so that names no curve, and the reader refused
    /// the solid. A centre both faces hold is four `f64`s down in `x` and
    /// three up in `z`, with the ball at `15.35` units.
    #[test]
    fn a_half_ball_on_a_long_thin_bar_placed_far_out_and_turned_is_read_back() {
        the_rim_comes_back_cut_from_its_plane(
            (0.125, 1955.404_019_307_245_3, 0.125),
            Vector::new(2_048.000_000_000_001, 0.0, -64.000_000_000_000_01),
            &a_small_turn_then_quarter_turns(
                Vector::new(
                    0.309_067_010_740_284_86,
                    -0.090_860_775_388_262_43,
                    -0.313_017_443_317_269_2,
                ),
                3.785_505_033_375_922_5e-6,
                1.0,
                3.0,
            ),
            [-4, 0, 3],
        );
    }

    /// Builds a bar with a dome on top ([`a_dome_on_a_bar`]), places it by
    /// `place` after `turn`, writes it and reads it back, and asks that the
    /// read solid is sound and that each use of the two halves of its rim is
    /// cut from the plane the rim was built in, placed, bit for bit; and that
    /// the rim was written centred `centred_out` `f64`s of `x`, `y` and `z`
    /// from where that plane meets the bar's axis.
    fn the_rim_comes_back_cut_from_its_plane(
        (radius, length, dome_radius): (f64, f64, f64),
        place: Vector,
        turn: &Isometry,
        centred_out: [i32; 3],
    ) {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(Scale::millimetres(10_000.0).expect("a working size"));
        let solid = a_dome_on_a_bar(&mut model, radius, length, dome_radius);
        let moved = Isometry::translation(place)
            .expect("a finite move")
            .compose(turn);
        model
            .place_solid(solid, &moved)
            .expect("the key came from this model");
        let went_in = what_its_rim_is_cut_from(&model);
        assert_eq!(went_in.len(), 4, "{went_in:?}");
        assert!(
            went_in.iter().all(|and| matches!(and, Surface::Plane(_))),
            "{went_in:?}"
        );
        let text = write(&model, "a dome on a long bar").expect("the dome is written");
        let file = StepFile::read(text.as_bytes()).expect("this writer's text parses");
        let back = Import::read(&file, &policy, &mut log)
            .unwrap_or_else(|why| panic!("the dome this writer wrote is refused: {why}"));
        back.model()
            .validate(&policy, &mut log)
            .expect("the dome read back is sound");
        assert_eq!(what_its_rim_is_cut_from(back.model()), went_in);

        let Surface::Plane(cut) = went_in[0] else {
            unreachable!("every half of the rim is cut from a plane, asked above");
        };
        let bar = model
            .solids()
            .next()
            .expect("the model holds one solid")
            .faces()
            .find_map(|face| {
                let Surface::Cylinder(bar) = face.surface() else {
                    return None;
                };
                Some(bar)
            })
            .expect("the bar's skin is a pipe");
        let on_the_axis = where_a_plane_crosses_a_pipe(&cut, &bar)
            .expect("the rim's plane crosses the bar")
            .centre
            .coordinates();
        let centre: [f64; 3] =
            core::array::from_fn(|axis| stepped(on_the_axis[axis], centred_out[axis]));
        assert!(
            text.contains(&triple(centre)),
            "the rim is not written centred at {centre:?}"
        );
    }

    /// A bar of `radius` standing on the origin along `z`, `length` tall, with
    /// a dome of `dome_radius` on top whose rim is the bar's top circle: a flat
    /// foot, the bar's skin with one straight seam, and a ball's cap with no
    /// pole on it. The rim is two halves, each cut from the plane `z = length`
    /// and picked by a half-space through the rim's centre.
    fn a_dome_on_a_bar(model: &mut Model, radius: f64, length: f64, dome_radius: f64) -> SolidKey {
        let below_the_rim = ((dome_radius - radius) * (dome_radius + radius)).sqrt();
        let top = Point::new(0.0, 0.0, length);
        let half_of_the_rim = |facing: Vector| EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(
                Plane::from_point_and_normal(top, Vector::new(0.0, 0.0, 1.0))
                    .expect("the z axis is a unit normal"),
            ),
            picked_by: [Picker::from(
                HalfSpace::through_a_point_facing(top, facing).expect("a facing with a length"),
            )]
            .into(),
        };
        let mut builder = SolidBuilder::new();
        let [foot, near, far] = [
            Point::new(radius, 0.0, 0.0),
            Point::new(radius, 0.0, length),
            Point::new(-radius, 0.0, length),
        ]
        .map(|at| builder.add_vertex(at).expect("a corner"));
        let [round_the_foot, up_the_seam, rim_in_front, rim_behind] = [
            (foot, foot, EdgeRun::RightRoundWhereItsFacesCross),
            (foot, near, EdgeRun::Straight),
            (near, far, half_of_the_rim(Vector::new(0.0, 1.0, 0.0))),
            (far, near, half_of_the_rim(Vector::new(0.0, -1.0, 0.0))),
        ]
        .map(|(from, to, run)| builder.add_edge(from, to, run).expect("an edge"));
        let [
            under,
            round_the_bar,
            up,
            behind_backwards,
            in_front_backwards,
            down,
            in_front,
            behind,
        ] = [
            (round_the_foot, Orientation::Reversed),
            (round_the_foot, Orientation::Forward),
            (up_the_seam, Orientation::Forward),
            (rim_behind, Orientation::Reversed),
            (rim_in_front, Orientation::Reversed),
            (up_the_seam, Orientation::Reversed),
            (rim_in_front, Orientation::Forward),
            (rim_behind, Orientation::Forward),
        ]
        .map(|(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
        let foot_wire = builder.add_wire(&[under]).expect("a wire");
        let bar_wire = builder
            .add_wire(&[
                round_the_bar,
                up,
                behind_backwards,
                in_front_backwards,
                down,
            ])
            .expect("a wire");
        let dome_wire = builder.add_wire(&[in_front, behind]).expect("a wire");
        let skin = [
            (
                Surface::Plane(
                    Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                        .expect("the z axis is a unit normal"),
                ),
                foot_wire,
            ),
            (
                Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a radius")),
                bar_wire,
            ),
            (
                Surface::Sphere(
                    Sphere::new(Point::new(0.0, 0.0, length - below_the_rim), dome_radius)
                        .expect("a radius"),
                ),
                dome_wire,
            ),
        ]
        .map(|(surface, wire)| {
            (
                builder.add_face(surface, &[wire]).expect("a face"),
                Orientation::Forward,
            )
        });
        builder.add_shell(&skin).expect("a shell");
        builder
            .build(model, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a sound dome on a bar")
    }

    /// What each use of an edge cut from a surface is cut from, face by face,
    /// in the model's one solid.
    fn what_its_rim_is_cut_from(model: &Model) -> Vec<Surface> {
        model
            .solids()
            .next()
            .expect("the model holds one solid")
            .faces()
            .flat_map(|face| face.wires().flat_map(WireView::coedges).collect::<Vec<_>>())
            .filter_map(|coedge| {
                let EdgeRun::PartOfWhereItCrosses { and, .. } = coedge.edge().run() else {
                    return None;
                };
                Some(and)
            })
            .collect()
    }

    /// A turn of `small` about `direction`, after `about_z` quarter turns about
    /// `z` that follow `about_x` about `x`.
    fn a_small_turn_then_quarter_turns(
        direction: Vector,
        small: f64,
        about_z: f64,
        about_x: f64,
    ) -> Isometry {
        let quarter = |axis: Axis, times: f64| {
            Isometry::rotation(&axis, times * FRAC_PI_2).expect("a finite turn")
        };
        Isometry::rotation(
            &Axis::new(Point::origin(), direction).expect("a direction with a length"),
            small,
        )
        .expect("a finite turn")
        .compose(&quarter(Axis::z(), about_z).compose(&quarter(Axis::x(), about_x)))
    }

    fn the_plane_its_seam_is_cut_from(model: &Model) -> Plane {
        let run = model
            .solids()
            .next()
            .expect("the model holds one solid")
            .faces()
            .next()
            .expect("a ball has one face")
            .wires()
            .flat_map(WireView::coedges)
            .next()
            .expect("a ball's one wire runs down its seam")
            .edge()
            .run();
        let EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(cut),
            ..
        } = run
        else {
            panic!("a ball's seam is cut from a plane, not {run:?}");
        };
        cut
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
    /// `1/√2`. `Plane::from_point_and_normal`, which the reader called for
    /// every `PLANE` until §313.3 was closed, moves it to the `f64` just above,
    /// and **no direction a file could hold reads back as it through that
    /// constructor**. That is why the reader keeps a written normal instead
    /// (`the_reader_keeps_a_normal_a_turn_left_short_of_length_one`), and why
    /// nothing the writer chose could have fixed it.
    ///
    /// That constructor's components are each the nearest `f64` to the exact
    /// unit ratio (`rcad-geom`'s `a_unit_component_is_the_nearest_f64_to_the_exact_ratio`),
    /// so a direction reads back as `(s, 0, s)` only if its unit vector is
    /// within half an ulp, `h = 2⁻⁵⁴`, of `s` in `x` and `z` and rounds to zero
    /// in `y`. The longest such vector is `(s + h, ≤2⁻¹⁰⁷⁵, s + h)`, and it is
    /// shorter than one: with `s = m·2⁻⁵³`, `2·(s + h)² < 1` is
    /// `2·(2m + 1)² < 2¹⁰⁸` in whole numbers, and the `y` term is far too
    /// small to close a gap of at least `2⁻¹⁰⁸`. So no unit vector rounds to
    /// this normal.
    #[test]
    fn no_direction_reads_back_as_a_normal_a_turn_left_short_of_length_one() {
        let long = core::f64::consts::FRAC_1_SQRT_2;
        let short = long.next_down();
        assert_eq!(
            format!("{short:?}"),
            "0.7071067811865475",
            "what the crash file held"
        );
        let read = Plane::from_point_and_normal(Point::origin(), Vector::new(short, 0.0, short))
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

    /// The same normal, written as the `DIRECTION` of a `PLANE` and read back
    /// through the reader's own function, keeps its bits, and the point the
    /// writer picks for it still carries its offset.
    #[test]
    fn the_reader_keeps_a_normal_a_turn_left_short_of_length_one() {
        let short = core::f64::consts::FRAC_1_SQRT_2.next_down();
        let written = Vector::new(short, 0.0, short);
        let read =
            plane_through(Point::new(3.0, -2.0, 5.0), written).expect("a direction with a length");
        assert_eq!(
            read.normal().components().map(f64::to_bits),
            written.components().map(f64::to_bits),
            "{read:?}"
        );

        let point = a_point_this_reader_reads_back_as(&read)
            .unwrap_or_else(|why| panic!("{read:?}: {why}"));
        let back = plane_through(point, read.normal()).expect("a finite point");
        assert_eq!(
            back.coefficients().map(f64::to_bits),
            read.coefficients().map(f64::to_bits),
            "{read:?} read back from {point:?} as {back:?}"
        );
    }

    /// Two planes the writer fuzzer's 2026-09-26 crash files
    /// `crash-7444ab6408bdd8fc9ca34f84cbf6fcd392681901` and
    /// `crash-848d6458b595df9f964acfc599a4153579e42d34` placed: facing along
    /// an axis, an ulp longer than one, which the reader keeps. The quotient
    /// `offset / nᵢ` misses by a couple of ulps and there is no second
    /// component to carry the rest, so the coordinate is stepped until it
    /// reads back; before it was, both were refused by name.
    #[test]
    fn a_normal_along_an_axis_an_ulp_too_long_is_written_at_a_point_it_reads_back_from() {
        let long = 1.0_f64.next_up();
        for (normal, offset) in [
            (Vector::new(0.0, 0.0, -long), 0.999_999_999_999_998_2),
            (Vector::new(long, 0.0, 0.0), -0.999_999_999_999_998_4),
        ] {
            let plane = Plane::from_point_and_normal_as_written(Point::origin(), normal)
                .and_then(|plane| plane.moved_to(offset))
                .expect("a finite offset along a normal with a length");
            assert_eq!(plane.normal(), normal, "the reader keeps this normal");
            let point = a_point_this_reader_reads_back_as(&plane)
                .unwrap_or_else(|why| panic!("{plane:?}: {why}"));
            let back = plane_through(point, plane.normal()).expect("a finite point");
            assert_eq!(
                back.coefficients().map(f64::to_bits),
                plane.coefficients().map(f64::to_bits),
                "{plane:?} read back from {point:?} as {back:?}"
            );
        }
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

    use super::{HALF_ANGLE_STEPS_SEARCHED, a_half_angle_that_reads_back_as};

    /// The slope the reader makes of a written half-angle: `cone_from` takes
    /// its tangent, as [`Cone::from_half_angle`] does.
    fn read_back(half_angle: f64) -> f64 {
        Cone::from_half_angle(Axis::z(), half_angle)
            .expect("a half-angle strictly inside a right angle")
            .slope()
    }

    /// Two of the four slopes the writer target draws, a power of two each so
    /// a cone's corner lies on it exactly, come back bit for bit from the
    /// angle written. Half does not come back from `atan` alone: it comes back
    /// `0.49999999999999994`, and the comparison in the writer target failed
    /// on that before the angle was searched (Q70).
    ///
    /// **One and two come back from no angle at all.** The `f64` nearest a
    /// quarter turn is below it, and its tangent rounds to the `f64` below
    /// one; the next angle up is above it, and its tangent rounds to the `f64`
    /// above one. Two falls between two neighbouring angles' tangents the same
    /// way. So a cone of either slope comes back an ulp or two off whatever is
    /// written, and the target compares its slope as a measurement
    /// (`docs/decisions.md` §366).
    #[test]
    fn each_slope_the_writer_target_draws_reads_back_from_the_angle_written() {
        assert_eq!(read_back(0.5_f64.atan()), 0.499_999_999_999_999_94);
        for slope in [0.25, 0.5] {
            let written = a_half_angle_that_reads_back_as(slope);
            assert_eq!(
                read_back(written).to_bits(),
                slope.to_bits(),
                "{slope} written as {written}"
            );
        }

        let quarter_turn = 1.0_f64.atan();
        assert_eq!(quarter_turn, core::f64::consts::FRAC_PI_4);
        assert_eq!(read_back(quarter_turn), 1.0_f64.next_down());
        assert_eq!(read_back(quarter_turn.next_up()), 1.0_f64.next_up());
        for slope in [1.0_f64, 2.0] {
            let first = slope.atan();
            let [below, at, above] = [first.next_down(), first, first.next_up()].map(read_back);
            assert!(below < at && at < above, "{slope}: {below} {at} {above}");
            assert!(
                (below < slope && slope < at) || (at < slope && slope < above),
                "{slope} is not strictly between two neighbouring angles' tangents: \
                 {below} {at} {above}"
            );
        }
    }

    /// Over a hundred thousand slopes from `1e-3` to `1e3`: how many come
    /// back from `atan`, how many from an angle the search finds, and how many
    /// from no angle at all — and for every one, that the slope comes back
    /// within what two neighbouring angles' tangents differ by, the bound the
    /// writer target's comparison holds a cone's slope to.
    ///
    /// **Where no angle is found, none exists**, given only that `tan` is
    /// faithful — off by less than an ulp of the true tangent — which no
    /// libm falls short of. The tangents of the angles searched are then
    /// strictly increasing and the slope lies strictly between two of them, so
    /// no angle in the window reads back as it, and every angle outside the
    /// window is further still: below it the tangent is below the window's
    /// least, above it above its most (`docs/decisions.md` §366).
    ///
    /// **It shows the limit and does not pin the search.** None of these slopes
    /// comes back from an angle the search finds and `atan` misses, so with the
    /// search taken out every count here is the same. Half is such a slope, and
    /// [`each_slope_the_writer_target_draws_reads_back_from_the_angle_written`]
    /// pins the search on it here, as does
    /// `a_cone_of_a_slope_some_half_angle_carries_comes_back_with_that_slope_to_the_bit`
    /// in `crates/rcad-step/tests/fuzzing/writer.rs` through the whole writer.
    #[test]
    fn most_slopes_over_one_come_back_from_no_half_angle() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1_u64 << 53) as f64
        };
        let (mut from_atan, mut found, mut over_one, mut none_over_one, mut none) = (0, 0, 0, 0, 0);
        let mut furthest_found = 0;
        let mut worst_share_of_the_reach: f64 = 0.0;
        for _ in 0..100_000 {
            let slope = 1e-3 + next() * (1e3 - 1e-3);
            let first = slope.atan();
            let written = a_half_angle_that_reads_back_as(slope);
            let back = read_back(written);
            let reach = (read_back(first.next_up()) - read_back(first.next_down())).abs();
            worst_share_of_the_reach = worst_share_of_the_reach.max((back - slope).abs() / reach);
            over_one += usize::from(slope > 1.0);
            if back == slope {
                let steps = (written.to_bits() as i64 - first.to_bits() as i64).unsigned_abs();
                if steps == 0 {
                    from_atan += 1;
                } else {
                    found += 1;
                    furthest_found = furthest_found.max(steps);
                }
                continue;
            }
            none += 1;
            none_over_one += usize::from(slope > 1.0);
            let mut angle = f64::from_bits(first.to_bits() - HALF_ANGLE_STEPS_SEARCHED as u64);
            let mut tangents = Vec::with_capacity(2 * HALF_ANGLE_STEPS_SEARCHED + 1);
            for _ in 0..=2 * HALF_ANGLE_STEPS_SEARCHED {
                tangents.push(read_back(angle));
                angle = angle.next_up();
            }
            assert!(
                tangents.windows(2).all(|pair| pair[0] < pair[1]),
                "{slope}: the tangents near atan are not increasing"
            );
            assert!(
                tangents[0] < slope && slope < tangents[2 * HALF_ANGLE_STEPS_SEARCHED],
                "{slope}: the window does not straddle the slope"
            );
        }
        println!(
            "100000 slopes from 1e-3 to 1e3: {from_atan} come back from atan, {found} from an \
             angle the search finds (furthest {furthest_found} steps), {none} from none \
             ({none_over_one} of the {over_one} over one); the worst comes back \
             {worst_share_of_the_reach} of two angles' reach off"
        );
        assert!(
            worst_share_of_the_reach <= 1.0,
            "a slope came back further off than two neighbouring angles' tangents differ"
        );
        assert!(
            furthest_found < HALF_ANGLE_STEPS_SEARCHED as u64 / 8,
            "an angle was found {furthest_found} steps out, near the end of the search"
        );
        assert!(
            none_over_one * 10 > over_one * 9,
            "only {none_over_one} of {over_one} slopes over one come back from no angle"
        );
    }

    /// The demonstration behind `Writing::surface`'s doc comment: a cone's
    /// slope is not written down — STEP has no field for a tangent, only a
    /// half-angle — so every cone this writer emits goes out as an angle and
    /// comes back through `tan` (in `cone_from`, and here through the public
    /// [`Cone::from_half_angle`](rcad_geom::Cone::from_half_angle) that does
    /// the same conversion). Through `atan` alone, as here, that round trip
    /// is not exact, which is why `Writing::surface` writes the angle
    /// `a_half_angle_that_reads_back_as` searches for instead.
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
