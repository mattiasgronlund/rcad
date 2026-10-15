use std::collections::HashMap;

use rcad_geom::{Cone, Deflection, Plane, Point, Surface, Vector};
use rcad_predicates::expansion::{compress, expansion_sum, scale_expansion, two_diff, two_product};
use rcad_predicates::{Policy, ProvenanceLog};

use crate::mesh::{Mesh, MeshTriangle};
use crate::view::FaceView;
use crate::{Error, FaceKey, Invalid, Result, SolidView};

/// How much space a solid takes up, how much skin it has, where it balances,
/// and how far each of those three could be out.
///
/// **Three answers in three different units, and each error is in the unit of
/// the answer it belongs to** — a cubed one, a squared one and a length
/// (`docs/decisions.md` §25.4). Cramming a volume error into an
/// [`Approximation`](rcad_predicates::Approximation), which is a length by
/// §15.2, is the units mistake §12.4 says this kernel loses money on.
///
/// For a solid whose faces are all flat, the divergence-theorem sum over its
/// own triangles is the answer and not a picture of it, so all that the three
/// errors carry is the rounding of those sums — exactly `0.0` where nothing
/// rounded, as on a cube with whole-number corners, and a rounding or so where
/// something did, as on a turned one (`docs/decisions.md` §338).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    volume: f64,
    volume_error: f64,
    surface_area: f64,
    area_error: f64,
    centroid: Point,
    centroid_wobble: f64,
}

impl Measurement {
    /// How much space the solid encloses.
    ///
    /// **Can be negative, but never by more than [`Measurement::volume_error`].**
    /// A curved solid as thin as a sliver can sum over its mesh to a little
    /// under zero while the true volume is a little over; the two quarter bends
    /// in `a_bend_placed_round_by_minus_two_and_a_copy_a_quarter_turn_back_overlap_in_a_sliver`
    /// leave a wedge of `5.4e-17` that sums to `-8.7e-17` at a deflection of
    /// `0.01`, with an error of `2.0e-15`. A sum further below zero than that is
    /// refused, as [`SolidView::measure`] says.
    #[must_use]
    pub fn volume(self) -> f64 {
        self.volume
    }

    /// How far [`Measurement::volume`] can be from the space the solid really
    /// encloses — a **distance**, and a **cubed** quantity in the model's own
    /// unit.
    ///
    /// **A distance and not a shortfall, because which way it is out depends on
    /// which way the surface bends.** The triangles a curved face is cut into
    /// are drawn through points on the surface itself, so on a round bar they
    /// lie inside the material and the sum over them falls short. A round
    /// **hole**'s wall bends the other way: the same chords cut into the block,
    /// so the mesh reports more material than there is — a cube of side 8 with a
    /// hole of radius 1 through it comes out `0.477` **over** at a deflection of
    /// `0.1`. `a_block_with_a_round_hole_measures_bigger_than_it_is` is that
    /// shape, and
    /// `a_measured_volume_sits_inside_the_published_bound_over_balls_and_bars`
    /// is where this number is checked against `|true − measured|` rather than
    /// against a signed shortfall.
    ///
    /// What is on the wrong side of the triangles either way is the shell
    /// between them and the surface — same shell, same thickness, opposite sign
    /// — and [`how_much_space_hides_under_a_flat_stand_in`] bounds it.
    ///
    /// A flat face of a curved solid has no shell, but its mesh points lie on
    /// its plane only as nearly as their coordinates can be written down, and
    /// a turned plane's rarely can. So it is charged the slab its triangles can
    /// stray through: [`how_far_the_points_stray_off`] its plane times its
    /// area, and the furthest any flat face strays times the slivers along its
    /// rim, as [`SolidView::skin_of`] says.
    ///
    /// For a wholly flat solid there is no such shell, and this is how far the
    /// rounded sum landed from the exact volume of the solid's own triangles,
    /// worked out rather than bounded in advance: `0.0` exactly when the sum
    /// is exact. A curved solid owes the same rounding of its sum over the
    /// mesh, worked out the same way, on top of the shell.
    /// `a_flat_solid_publishes_the_rounding_of_its_measure.rs` holds both
    /// against the volume of the triangles worked out exactly. Never negative.
    #[must_use]
    pub fn volume_error(self) -> f64 {
        self.volume_error
    }

    /// The total area of all its faces.
    #[must_use]
    pub fn surface_area(self) -> f64 {
        self.surface_area
    }

    /// How far [`Measurement::surface_area`] can be from the skin the solid
    /// really has — a **distance**, and a **squared** quantity in the model's
    /// own unit.
    ///
    /// Two things are charged and both are counted:
    /// [`how_much_more_surface_there_can_be`] for the curved faces, whose true
    /// surface bulges away from the flat triangles standing in for it, and
    /// [`rim_of_the_flat_faces`] for the flat ones, whose meshed outline is a
    /// polygon inscribed in a curve.
    ///
    /// **A distance for the same reason [`Measurement::volume_error`] is, and
    /// the two terms are what pull opposite ways.** On a round bar both are
    /// short of the truth. Round a **hole**, the flat face's inscribed polygon
    /// takes *less* away than the circle it stands for, so that face is meshed
    /// too large, while the hole's wall is still meshed too small. Which of the
    /// two wins is the hole's depth: a cube of side 8 with a hole of radius 1
    /// straight through comes out `0.128` under at a deflection of `0.1`, and an
    /// 8 × 8 plate half a unit thick with the same hole comes out `0.127`
    /// **over**. `a_thin_plate_with_a_hole_measures_more_skin_than_it_has` is
    /// both of those measured.
    ///
    /// For a wholly flat solid, how far each face's rounded area and their
    /// rounded total landed from the exact areas of the faces' own fans:
    /// `0.0` exactly when every one of those came out exact. Never negative.
    #[must_use]
    pub fn area_error(self) -> f64 {
        self.area_error
    }

    /// The point the solid would balance on, if it were of even density.
    #[must_use]
    pub fn centroid(self) -> Point {
        self.centroid
    }

    /// How far [`Measurement::centroid`] could be from where the solid really
    /// balances — a **length**, in the model's own unit.
    ///
    /// The one of the three that is a length, so the one that is comparable
    /// with [`Policy::max_wobble`] (§25.4). See [`SolidView::measure`] for what
    /// that comparison is worth and who makes it.
    ///
    /// The material the mesh gets wrong — missing on a bar, added round a hole
    /// — is no more than the shell [`Measurement::volume_error`] charges, and
    /// none of it is further from the centroid than the solid reaches, so
    /// moving all of it to the far side shifts the balance point by at most
    /// `shell / volume` of that reach. On top of that, how far the rounded
    /// centroid landed from where the mesh's own triangles balance exactly,
    /// worked out as it is for a flat solid below;
    /// `a_flat_solid_publishes_the_rounding_of_its_measure.rs` holds a curved
    /// solid's against that point.
    ///
    /// For a wholly flat solid there is no such material, and this is how far
    /// the rounded centroid landed from where the solid's own triangles balance
    /// exactly — each tetrahedron they make with the hub weighed by its signed
    /// volume, the same triangles the volume is the exact sum of — worked out
    /// rather than bounded in advance, as the volume and area errors are:
    /// `0.0` exactly when the centroid came out on the nose, as on a cube with
    /// whole-number corners (`docs/decisions.md` §338.5). Never negative.
    ///
    /// **Infinite when the volume is not clear of its own error**, because
    /// `shell / volume` then bounds nothing: a solid that may enclose no space
    /// at all has no balance point to be near. Where the volume is not even
    /// above zero, the moment cannot be divided by it, and
    /// [`Measurement::centroid`] is where the mesh's skin balances instead —
    /// a point near the solid, and no more.
    #[must_use]
    pub fn centroid_wobble(self) -> f64 {
        self.centroid_wobble
    }
}

/// How much area one face of a solid has, where that area balances, and how
/// far each of those could be out.
///
/// **Two answers in two units, and each error is in its own answer's unit** —
/// a squared one and a length, as [`Measurement`] keeps them (`docs/decisions.md`
/// §25.4).
///
/// It is [`SolidView::measure`] cut down to one face, not a second measurer:
/// the same triangles, the same arithmetic and the same charges, so the faces
/// of a solid add up to its [`Measurement::surface_area`] **bit for bit** when
/// they are added in the order [`SolidView::faces`] hands them out, starting
/// from `0.0`. `the_faces_of_a_solid_add_up_to_its_skin_bit_for_bit` is that
/// law over cubes, bars, balls, cones, a bend and a fuse.
///
/// For a face of a solid whose faces are all flat, the area error is the
/// rounding of the face's own fan and nothing else, for the reason
/// [`Measurement`] gives — and what the truth is, is the area of the corners
/// the solid *holds*. A cube turned and put a million units out holds corners
/// the turn rounded, so its faces are `1.25e-10` off `side²`, which is where
/// the corners are; the area error is how far the measured area is from the
/// area those corners really bound, a rounding or so.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceMeasurement {
    area: f64,
    area_error: f64,
    centroid: Point,
    centroid_wobble: f64,
}

impl FaceMeasurement {
    /// The face's area.
    #[must_use]
    pub fn area(self) -> f64 {
        self.area
    }

    /// How far [`FaceMeasurement::area`] can be from the area the face really
    /// has — a **distance**, and a **squared** quantity in the model's own
    /// unit.
    ///
    /// The one charge [`Measurement::area_error`] makes for this face and no
    /// other: [`how_much_more_surface_there_can_be`] on a curved face, what a
    /// cone face reaching its own apex costs on one of those, and the
    /// deviation times [`rim_of_the_flat_faces`] on a flat face of a curved
    /// solid — the sides where this face meets another, whose meshed outline is
    /// a polygon inscribed in a curve. So the faces' errors add up to the
    /// solid's, to within the rounding of adding them in a different order.
    ///
    /// **How big it is against the area, measured** at a deflection of `0.01`:
    /// `3.32e-3` of a round bar's cap and `3.33e-3` of its wall at radius
    /// three, `6.70e-3` of a ball of radius three, `1.97e-2` and `6.56e-3` of a
    /// frustum's caps of radius one and three and `1.78e-2` of its side,
    /// `6.63e-3` of a pointed cone's cap of radius two and `1.81e-2` of its
    /// side, and `2.00e-2` of a bend's caps and `2.03e-2` of its tube of radius
    /// one. A curved face costs about twice the deviation over its radius and a
    /// flat cap about the deviation times its rim over its area;
    /// `what_a_faces_area_error_is_against_its_area` pins each to half a
    /// percent.
    ///
    /// For a face of a wholly flat solid, how far the rounded area landed from
    /// the exact area of the face's own fan: `0.0` exactly when it did not
    /// round. Never negative.
    #[must_use]
    pub fn area_error(self) -> f64 {
        self.area_error
    }

    /// The point the face's surface would balance on, if it were a skin of
    /// even thickness.
    #[must_use]
    pub fn centroid(self) -> Point {
        self.centroid
    }

    /// How far [`FaceMeasurement::centroid`] could be from where the face
    /// really balances — a **length**, in the model's own unit.
    ///
    /// # The derivation
    ///
    /// Write `A` for the area of the face's triangles, `E` for
    /// [`FaceMeasurement::area_error`], `d` for the mesh's
    /// [`Mesh::deviation`] and `reach` for how far the triangles reach from the
    /// centroid they give, plus `d`. Pair each point of the triangles with the
    /// point of the surface the area bound already pairs it with — radially
    /// out on to a ball, straight out from a pipe's axis, along the normal of
    /// a cone or a tube — and let `J` be how much that pairing stretches area.
    /// Then the true face's first moment about the measured centroid differs
    /// from the triangles' by two things: every paired point moved by at most
    /// `d`, which is at most `d · A_true`; and wherever `J` is not one, area
    /// that has nothing to pair with, none of it further than `reach` out,
    /// which is at most `reach · ∫|J − 1|`. Dividing by the true area, which is
    /// at least `A − E`:
    ///
    /// ```text
    ///     wobble = moved + reach · unpaired / (A − E)
    /// ```
    ///
    /// - **A curved face**: `moved = d`, and `unpaired = 3E`. The area bound's
    ///   own derivation is that `J` never exceeds the factor it charges, so
    ///   the part of `J` over one adds up to at most `E`; the part under one is
    ///   that minus the net difference in area, which is at most `E` again; so
    ///   `∫|J − 1| ≤ 2E + E`.
    /// - **A cone face that reaches its apex** has no pairing below the split
    ///   [`what_a_cone_tip_costs`] draws, so everything down there is charged
    ///   as unpaired on both sides: the cone's whole tip below the split and
    ///   the triangles' share of it, twice over — once as area that pairs with
    ///   nothing and once in the net difference above the split.
    /// - **A flat face of a curved solid**: each point moves straight on to the
    ///   face's plane, so `moved` is [`how_far_the_points_stray_off`] it — `0`
    ///   where every point lies on the plane exactly, and a rounding or so on
    ///   a turned plane, whose points rarely can; and what is
    ///   unpaired is the slivers between the inscribed outline and the curve,
    ///   at most `E` of them and within `d` of a side of the mesh.
    ///
    /// Infinite whenever `E` is — a mesh cut a whole radius out — or is not
    /// smaller than `A`, because then nothing bounds the true area away from
    /// zero and the division has nothing under it. A deflection that coarse is
    /// a legitimate request (§23.5) and this is what it costs.
    ///
    /// **The pairing is the area bound's, so this rests on that bound.** Where
    /// the pairing misses a sliver at the face's edge, the area bound misses
    /// it too; `the_middle_of_each_face_is_within_its_wobble` holds this number
    /// against the textbook centroid of balls, bars, a frustum, a pointed cone
    /// and a bend, at four deflections, at the origin and turned a million
    /// units out.
    ///
    /// **It is loose, and by how much is measured**: over those 96 faces the
    /// closest a centroid came was `0.0022` of its wobble, so about five
    /// hundred times what the mesh really got wrong. The meshes of these
    /// symmetric faces are symmetric too and put the middle almost exactly
    /// where it belongs; the `reach · 3E` term cannot know that, and it is
    /// what dominates. A bound on how lopsided a face's triangles can be would
    /// tighten it; nothing here has one.
    ///
    /// For a face of a wholly flat solid, how far the rounded centroid landed
    /// from where the face's own fan balances exactly, each triangle weighed by
    /// its signed shadow on the fan's exact vector area: `0.0` exactly when it
    /// did not round. A turned face's corners are not quite in one plane, so
    /// which middle is exact needed choosing, and
    /// [`how_far_the_middle_of_its_fan_rounded`] says why this one
    /// (`docs/decisions.md` §338.5). Never negative.
    #[must_use]
    pub fn centroid_wobble(self) -> f64 {
        self.centroid_wobble
    }
}

impl SolidView<'_> {
    /// Measure the solid: volume, surface area and centroid in one pass, each
    /// with how far out it could be.
    ///
    /// Straight out of the divergence theorem. Each triangle and the hub make a
    /// tetrahedron whose signed volume is a determinant; add them all up and the
    /// parts outside the solid cancel, because the triangles form a closed
    /// surface. Faces used the wrong way round come out negative, which is why
    /// [`Orientation::sign`] has to be exactly ±1 — and it is.
    ///
    /// # Two paths, and a polyhedron still takes the exact one
    ///
    /// A solid whose faces are **all flat** is measured over the triangles
    /// [`FaceView::for_each_triangle`] hands out, exactly as it was in M0:
    /// `deflection` is ignored, no mesh is built, nothing is logged, and the
    /// value of every answer is bit for bit what M0 gave. The three errors are
    /// then only the rounding of those sums, worked out exactly, so `0.0`
    /// wherever nothing rounded (§338). That is not an optimisation.
    /// Every metamorphic property in the fuse suite — inclusion–exclusion,
    /// idempotence, absorption — reads these numbers, and putting them
    /// downstream of a triangulator is the schedule risk §23.7 named before this
    /// work started. `a_flat_solids_measurement_is_bit_for_bit_what_it_was` is
    /// the test that says it did not happen.
    ///
    /// A solid with a curved face has no such fan — its caps fan into no
    /// triangles at all and its wall into two with no area — so it is measured
    /// over [`SolidView::mesh`], and the three error terms say how much of the
    /// answer the triangles could be hiding. The volume's and the centroid's
    /// add how far their own sums over the mesh rounded, worked out as the flat
    /// path works it out.
    ///
    /// # Which knob decides what
    ///
    /// `deflection` decides how fine the mesh is, and a coarse one is a request
    /// rather than a failure (§23.5). `policy` decides the one thing §23.5 gives
    /// it: whether a deflection under what the arithmetic can reach is refused
    /// or tagged, which is [`SolidView::mesh`]'s business and is why both travel
    /// through to it.
    ///
    /// **What `policy` deliberately does not do is refuse a measurement for
    /// being meshed.** [`Measurement::centroid_wobble`] is the one term
    /// comparable with [`Policy::max_wobble`], and that budget is 64 roundings
    /// at the working size — around `1e-12` for a 100 mm part. The finest mesh
    /// this kernel will make of a ball leaves a wobble many orders above it, so
    /// judging one against the other would refuse *every* curved solid at
    /// *every* deflection, and this signature would have replaced one blanket
    /// refusal with another. It would also let the deflection decide what the
    /// policy certifies, which is the conflation §23.5 exists to stop. The
    /// comparison is the caller's to make, on a number that is handed back for
    /// exactly that;
    /// `judging_a_meshed_centroid_against_the_policy_would_refuse_every_ball` is
    /// the demonstration rather than the paragraph.
    ///
    /// [`Orientation::sign`]: rcad_geom::Orientation::sign
    /// [`FaceView::for_each_triangle`]: crate::FaceView::for_each_triangle
    ///
    /// # Errors
    ///
    /// [`Error::VolumeTooLargeToMeasure`] or
    /// [`Error::SurfaceAreaTooLargeToMeasure`] when that answer is a real
    /// number the solid genuinely has and `f64` has no room for — a cube of
    /// side `1e300` encloses `1e900`. The solid is sound; the type is the
    /// limit, so this is bad input rather than a broken model (§4.1).
    ///
    /// For a curved solid, whatever [`SolidView::mesh`] refuses:
    /// [`Error::DeflectionBelowWhatAMeshCanReach`],
    /// [`Error::MoreMeshPointsThanASolidWillHold`], [`Error::CannotMeshAFace`]
    /// or [`Error::Undecidable`].
    ///
    /// [`Invalid::VolumeIsNotPositive`] when the solid encloses a negative
    /// amount of space, or none. A wholly flat solid is refused for a sum of
    /// zero or less — the same refusal
    /// [`Model::validate`](crate::Model::validate) gives for the same bytes,
    /// with the same number in it. A curved solid is refused only for a sum of
    /// at most minus its [`Measurement::volume_error`], because a sliver's mesh
    /// can sum a little under zero round a true volume a little over it; a sum
    /// within its error of zero is answered, as [`Measurement::volume`] says.
    /// `validate` does not yet charge a curved solid's flat faces what
    /// [`SolidView::skin_of`] charges them, so it can still refuse a sliver
    /// this answers (Q2089).
    ///
    /// **This used to be an assert, and §20.1 settled it that way.** The
    /// argument then was that the only route to an inside-out solid was a bug
    /// in this crate, so a crash beat a corruption. `Model` is `Deserialize`,
    /// which is a second route: a stranger's bytes can carry a solid whose
    /// faces point inwards, `validate` already answers `Err` for them, and a
    /// caller's bytes may not panic. `measuring_a_solid_read_back_inside_out_is_refused_rather_than_crashing`
    /// is the demonstration.
    ///
    /// # Panics
    ///
    /// For a curved solid, everything [`SolidView::mesh`] panics on.
    pub fn measure(
        self,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Measurement> {
        if self.is_wholly_flat() {
            return self.measured_over_its_own_fan();
        }
        let mesh = self.mesh(deflection, policy, log)?;
        self.measured_over(&mesh)
    }

    /// The answer for a polyhedron, summed over the triangles its faces fan
    /// into, with how far the rounded sums landed from the exact ones.
    ///
    /// No quadrature. Planar faces do not need any (decisions.md §3.1). The
    /// triangles come in a fixed order, so the sums are bit for bit repeatable.
    fn measured_over_its_own_fan(self) -> Result<Measurement> {
        let hub = self.hub();
        let shifts = self.axis_shifts(hub);
        let (six_volumes, moment, fan) = self.volume_and_moment(hub, shifts);

        let whole_shift = shifts.iter().sum();
        let sixth = six_volumes / 6.0;
        let volume = shift(sixth, whole_shift);
        if !volume.is_finite() {
            return Err(Error::VolumeTooLargeToMeasure);
        }
        if volume <= 0.0 {
            return Err(Invalid::VolumeIsNotPositive { volume }.into());
        }
        let (surface_area, area_error) = self.surface_area();
        if !surface_area.is_finite() {
            return Err(Error::SurfaceAreaTooLargeToMeasure);
        }

        let balance = [0, 1, 2].map(|axis| shift(moment[axis] / (4.0 * six_volumes), shifts[axis]));
        let centroid = hub + Vector::from(balance);
        let (exact_six_volumes, exact_moment) = volume_and_moment_as_expansions(&fan);
        let four_times_the_six_volumes = packed(&multiplied(&exact_six_volumes, &[4.0]));
        Ok(Measurement {
            volume,
            volume_error: how_far_the_volume_rounded(
                &exact_six_volumes,
                sixth,
                volume,
                whole_shift,
            ),
            surface_area,
            area_error,
            centroid,
            centroid_wobble: how_far_the_middle_is_from(
                centroid,
                hub,
                shifts,
                &exact_moment,
                &four_times_the_six_volumes,
            ),
        })
    }

    /// The same three sums over a mesh, plus how much of the solid the mesh
    /// leaves out.
    ///
    /// The arithmetic is the fan's: hang the sums from a corner rather than the
    /// origin, shrink each axis by a power of two, and undo both on the way out.
    /// [`SolidView::volume_and_moment`] is where that is derived, and every word
    /// of it applies here — the triangles are a different set, not a different
    /// kind of thing.
    ///
    /// **The volume's error is the space the mesh hides and the rounding of its
    /// sum, both, and so is the centroid's wobble.** The space hidden is the
    /// shell under the curved faces and the slab the flat faces' points stray
    /// through, both from [`SolidView::skin_of`]. The rounding is worked out
    /// as the flat path works it out, from the mesh's own triangles summed
    /// exactly. A solid whose curved faces are a hair of skin hides less space
    /// than one rounding of the sum, and published that space alone: the block
    /// less the cone in `a_flat_solid_publishes_the_rounding_of_its_measure.rs`
    /// measured `1152.0000000000007` with an error of `5.7e-19`, and its mesh
    /// encloses `8.4e-14` less; its centroid came out `1.2e-15` from where the
    /// mesh balances, with a wobble of `5.1e-21`.
    ///
    /// **The sign is judged against that error, so the error comes first.** A
    /// sum at most minus the error is a solid that encloses no space, or less,
    /// however the mesh is out, and is refused. A sum above that and not
    /// above zero is answered as it is, with the skin's middle for a centroid
    /// and an infinite wobble, since a moment divided by nothing balances
    /// nowhere.
    fn measured_over(self, mesh: &Mesh) -> Result<Measurement> {
        let hub = hub_of(mesh);
        let shifts = axis_shifts_over(mesh, hub);
        let SumsOverAMesh {
            six_volumes,
            moment,
            fan,
            ..
        } = volume_and_moment_over(mesh, hub, shifts);

        let whole_shift = shifts.iter().sum();
        let sixth = six_volumes / 6.0;
        let volume = shift(sixth, whole_shift);
        if !volume.is_finite() {
            return Err(Error::VolumeTooLargeToMeasure);
        }
        let skin = self.skin_of(mesh);
        if !skin.area.is_finite() {
            return Err(Error::SurfaceAreaTooLargeToMeasure);
        }
        let (exact_six_volumes, exact_moment) = volume_and_moment_as_expansions(&fan);
        let volume_error = upward(
            skin.volume_error
                + how_far_the_volume_rounded(&exact_six_volumes, sixth, volume, whole_shift),
        );
        if volume <= -volume_error {
            return Err(Invalid::VolumeIsNotPositive { volume }.into());
        }
        if volume <= 0.0 {
            return Ok(Measurement {
                volume,
                volume_error,
                surface_area: skin.area,
                area_error: skin.area_error,
                centroid: middle_of(mesh, mesh.triangles()),
                centroid_wobble: f64::INFINITY,
            });
        }

        let balance = [0, 1, 2].map(|axis| shift(moment[axis] / (4.0 * six_volumes), shifts[axis]));
        let centroid = hub + Vector::from(balance);
        let four_times_the_six_volumes = packed(&multiplied(&exact_six_volumes, &[4.0]));
        let centroid_wobble = if volume <= volume_error {
            f64::INFINITY
        } else {
            upward(
                (skin.volume_error / volume) * how_far_the_solid_reaches_from(mesh, centroid)
                    + how_far_the_middle_is_from(
                        centroid,
                        hub,
                        shifts,
                        &exact_moment,
                        &four_times_the_six_volumes,
                    ),
            )
        };
        Ok(Measurement {
            volume,
            volume_error,
            surface_area: skin.area,
            area_error: skin.area_error,
            centroid,
            centroid_wobble,
        })
    }

    /// How much skin the mesh has, and how much of the solid's own it misses.
    ///
    /// One walk over the faces rather than three, because the error terms are
    /// all a share of the very area being summed.
    ///
    /// **A flat face owes space too, once it is turned.** Its mesh points lie on
    /// its plane only as nearly as their coordinates can be written down, and
    /// a turned plane's rarely can: the caps of the sliver in
    /// `a_bend_placed_round_by_minus_two_and_a_copy_a_quarter_turn_back_overlap_in_a_sliver`
    /// are meshed up to `4.3e-16` off their planes, and the space between those
    /// triangles and their planes moved the sum by anywhere from `-4.9e-16` to
    /// `3.2e-16` (Q2087) against a wedge of `5.4e-17`. So each flat face is charged
    /// [`how_far_the_points_stray_off`] its plane times its area, the slab its
    /// triangles can stray through. The slivers between a flat face's
    /// inscribed outline and its curve are inside the neighbouring curved
    /// face's shell, as [`how_much_space_hides_under_a_flat_stand_in`] says,
    /// but that shell now ends at points up to the same distance off the
    /// plane, and those slivers are at most the deviation times
    /// [`rim_of_the_flat_faces`] in area, so that much again is charged times
    /// the furthest any flat face strays.
    fn skin_of(self, mesh: &Mesh) -> Skin {
        let bends = bend_of_each_face(self, mesh);
        let planes: HashMap<FaceKey, Plane> = self
            .faces()
            .filter_map(|face| face.surface().as_plane().map(|plane| (face.key(), *plane)))
            .collect();
        let deviation = mesh.deviation().magnitude();
        let mut skin = Skin {
            area: 0.0,
            area_error: 0.0,
            volume_error: 0.0,
        };
        let mut furthest_off_a_plane = 0.0f64;
        for (face, triangles) in each_face_and_its_triangles(mesh) {
            let area = area_of(mesh, triangles);
            skin.area += area;
            let bend = bends[&face];
            if let Some(charge) = bend.tip_charge {
                skin.area_error += charge.more_surface;
                skin.volume_error += charge.hidden_space;
                continue;
            }
            let Some(radius) = bend.radius else {
                let plane = planes
                    .get(&face)
                    .expect("a face that does not bend lies on a plane");
                let off = how_far_the_points_stray_off(mesh, triangles, plane);
                skin.volume_error += off * area;
                furthest_off_a_plane = furthest_off_a_plane.max(off);
                continue;
            };
            skin.area_error += how_much_more_surface_there_can_be(area, radius, deviation);
            skin.volume_error +=
                how_much_space_hides_under_a_flat_stand_in(area, radius, deviation);
        }
        let rim = rim_of_the_flat_faces(mesh, |face| bends[&face].radius.is_none());
        skin.area_error += deviation * rim;
        skin.volume_error += furthest_off_a_plane * deviation * rim;
        skin
    }

    /// The signed volume without the refusals, for
    /// [`Model::validate`](crate::Model::validate) to complain about. Infinite
    /// when the solid is too large to measure, which is a different complaint
    /// from a negative one and `validate` keeps the two apart.
    ///
    /// The fan, so this is for a wholly flat solid. A curved one has no fan and
    /// `validate` reads [`signed_volume_over`] instead.
    pub(crate) fn signed_volume(self) -> f64 {
        let hub = self.hub();
        let shifts = self.axis_shifts(hub);
        let (six_volumes, _, _) = self.volume_and_moment(hub, shifts);
        shift(six_volumes / 6.0, shifts.iter().sum())
    }

    /// Six times the signed volume, and the first moment about the hub, both in
    /// the units `shifts` picked rather than the model's.
    ///
    /// **The offsets are from a corner of the solid, not the world origin.**
    /// The sum is the same in exact arithmetic whatever the apex is, and wildly
    /// different in `f64`: hung from the origin, a solid `2^m` away adds up
    /// terms of size `2^{3m}` to reach a volume of order one, so past about
    /// `m = 28` the spacing of the terms is wider than the answer and what
    /// comes back has no correct digits and can be negative. Hung from a corner
    /// of the solid, every term is the size of the solid, and the three
    /// subtractions are exact by Sterbenz's lemma whenever the coordinates
    /// share an exponent — which is the case that used to be worst. A rescale
    /// does not stand in for this: it moves where the terms sit and cannot put
    /// back digits that cancellation has already taken.
    ///
    /// Every offset is then between one and two, so no triple product here can
    /// overflow or fall subnormal whatever the solid's shape. The overflow that
    /// used to happen inside this loop now happens in [`shift`] on the way out,
    /// where it means the answer does not fit rather than that the sum went
    /// wrong.
    ///
    /// # The shift is no defence against cancellation, and the sign needs one
    ///
    /// **The sign of this sum says what a shell *is*.** The fuse's
    /// `nest_the_shells` sorts a shell into a lump or a void of one by it, and
    /// [`Model::validate`](crate::Model::validate) and
    /// [`SolidView::measure`] call a negative one
    /// [`Invalid::VolumeIsNotPositive`]. A rounded comparison against zero
    /// deciding a shape is `docs/decisions.md` §49.8's class, and this is the
    /// member of it that fails into a wrong answer rather than a refusal.
    ///
    /// The paragraph above defends against being far from the origin. It does
    /// nothing against cancellation, and **the shift cannot be made to**: every
    /// term of a `3 × 3` determinant carries one factor from each axis, so
    /// shrinking an axis shrinks every term by exactly the same power of two
    /// and the sum comes back bit for bit what it was, scaled. That is what
    /// makes the shift free, and it is the same fact that makes it useless
    /// here — it moves where the terms sit and never how much of each other
    /// they eat. It does rescue a solid thin along an *axis*, because there
    /// every term is as small as the volume already. A solid thin along a
    /// *diagonal* has all three axis extents at its full size, terms of that
    /// size cubed, and a volume the terms must cancel down to reach.
    /// `a_flat_solid_whose_volume_signs_the_wrong_way.rs` is a rod `1e-9`
    /// across down `(1, 1, 1)`, with nothing near-parallel anywhere in it,
    /// whose `1e-18` of space used to measure `-4.5e-18`.
    ///
    /// So the rounded sum stands when a forward bound proves its own sign, and
    /// [`six_volumes_exactly`] answers when it does not. **The exact one always
    /// decides.** The corners are `f64` triples and every step from them to the
    /// sum is `+`, `-` and `×`, which dyadic fractions are closed under, so
    /// there is no undecided case to have here — unlike a predicate over a
    /// corner that is only known to lie in a box (§2.3). The one thing that
    /// can still come back without a sign is an overflow, and that is the
    /// answer not fitting rather than the sum going wrong, so it is handed
    /// straight back for [`shift`]'s callers to refuse.
    ///
    /// The moment is left rounded. It is a position, and where a solid
    /// balances is [`Measurement::centroid`], which nothing branches on.
    ///
    /// The third thing handed back is every triangle's offsets, exactly, which
    /// is what [`volume_and_moment_as_expansions`] needs to say how far the
    /// rounded sums landed from the exact ones.
    fn volume_and_moment(self, hub: Point, shifts: [i32; 3]) -> (f64, [f64; 3], ExactFan) {
        let mut fan: Vec<[[f64; 2]; 9]> = Vec::new();
        let mut six_volumes = 0.0;
        let mut how_big_the_terms_are = 0.0;
        let mut moment = [0.0f64; 3];
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                let exact = corners.map(|corner| exactly_how_far_from(hub, corner, shifts));
                let [a, b, c] = exact.map(the_rounded_part_of);
                let six_volume = a.dot(b.cross(c));
                six_volumes += six_volume;
                how_big_the_terms_are +=
                    across_the_axes(a) * across_the_axes(b) * across_the_axes(c);
                let offsets = [a, b, c].map(|offset| offset.components());
                for (axis, total) in moment.iter_mut().enumerate() {
                    *total += six_volume * (offsets[0][axis] + offsets[1][axis] + offsets[2][axis]);
                }
                fan.push(core::array::from_fn(|slot| exact[slot / 3][slot % 3]));
            });
        }

        // Every triple product costs at most eight roundings against the sum of
        // the six terms it is made of, and the running total one more each time
        // it is added to. `across_the_axes` dominates that six-term sum, so
        // this covers the lot with room to spare — and spare room only ever
        // costs the exact pass being run when it need not have been.
        let slack = (fan.len() as f64 + 8.0) * f64::EPSILON * how_big_the_terms_are;
        if six_volumes.abs() > slack || !how_big_the_terms_are.is_finite() {
            return (six_volumes, moment, fan);
        }
        (six_volumes_exactly(&fan), moment, fan)
    }

    /// The total area of the faces, each measured in a scale of its own, and
    /// how far that total can be from the area the faces' fans really have.
    ///
    /// **A different scaling from the one the volume uses, because a different
    /// thing is being measured.** A determinant scales by the product of its
    /// three column scales, so the volume can afford one power of two per axis;
    /// a length does not, because `x² + y² + z²` only comes apart if every
    /// component was scaled the same way. So each face is measured whole, in a
    /// single power of two.
    ///
    /// **Per face rather than per solid**, because the alternative loses small
    /// faces. A tower one unit across and `1e200` tall scaled to its own height
    /// would push its top and bottom to `1e-400`, which is zero, and the two
    /// units of area they carry would vanish. Measured in its own scale, each
    /// face keeps every bit it had.
    ///
    /// **The error is each face's own rounding and the rounding of adding them
    /// up**, both worked out rather than bounded in advance: the faces' areas
    /// are added again exactly, and what the running `f64` total dropped is the
    /// difference. So a solid whose faces all come out exact, and whose total
    /// is exact, owes exactly `0.0`.
    fn surface_area(self) -> (f64, f64) {
        let mut total = 0.0;
        let mut exactly = Vec::new();
        let mut error = 0.0;
        for face in self.faces() {
            let fan = OwnFan::of(face);
            total += fan.area;
            exactly = added(&exactly, &[fan.area]);
            error = upward(error + fan.area_error);
        }
        let dropped = added(&exactly, &[-total]);
        (total, upward(error + at_most(&dropped)))
    }

    /// One power of two per axis, so the widest corner offset along each lands
    /// between one and two.
    ///
    /// **Exact powers of two, which is what makes this free.** Scaling an axis
    /// scales every term of the determinant that uses it by the same amount,
    /// and floating-point addition commutes with an exact scaling, so the sums
    /// come back bit for bit what they would have been — and so does what
    /// [`shift`] undoes them to. `docs/decisions.md` §18: a rescale is a
    /// technique and not only an oracle.
    ///
    /// **One per axis rather than one for the solid**, because a solid need not
    /// be the same size in all three directions. A tower one unit across and
    /// `1e200` tall, scaled by its height, would have its two short axes land
    /// at `2^-664`, and their product with the third is `2^-1328` — zero, and a
    /// volume of nothing for a tower that plainly holds some.
    fn axis_shifts(self, hub: Point) -> [i32; 3] {
        let mut widest = [0.0f64; 3];
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                for corner in corners {
                    let offsets = (corner - hub).components();
                    for (axis, so_far) in widest.iter_mut().enumerate() {
                        *so_far = so_far.max(offsets[axis].abs());
                    }
                }
            });
        }
        widest.map(shift_for)
    }

    /// The corner the divergence-theorem sums hang from: the one with the
    /// smallest coordinates, `x` first.
    ///
    /// **A corner of the solid rather than the world origin**, for the reason
    /// [`Self::volume_and_moment`] gives, and **the smallest one rather than
    /// the first one the arena hands over**, because the arena's order is not
    /// part of the answer. Two routes through a fuse can reach the same solid
    /// with its faces in different orders, and a measurement that depended on
    /// which face came first would come back different for a solid that is the
    /// same — the property `fuses_that_must_agree.rs` calls decomposition
    /// invariance.
    ///
    /// The origin when the solid has no triangles at all. Nothing is summed in
    /// that case and the callers refuse the answer anyway.
    fn hub(self) -> Point {
        let mut smallest: Option<[f64; 3]> = None;
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                for corner in corners {
                    let corner = corner.coordinates();
                    if smallest.is_none_or(|so_far| lexicographically_smaller(corner, so_far)) {
                        smallest = Some(corner);
                    }
                }
            });
        }
        Point::from(smallest.unwrap_or([0.0; 3]))
    }

    /// Measure one face of this solid: its area and where it balances, each
    /// with how far out it could be.
    ///
    /// # Why it hangs off the solid and not off the face
    ///
    /// **A curved face cannot be meshed on its own.** A round bar's cap is
    /// bounded by a circle, and a plane has nothing to say about where a circle
    /// drawn on it goes: the circle is walked by the wall on the other side of
    /// it, and the cap reads the points the wall put there
    /// ([`SolidView::mesh`], `docs/decisions.md` §25.7). So what the cap's
    /// triangles are depends on its neighbours, and those are only reachable,
    /// already moved into place, through the solid the face sits in — the
    /// reason §3.2 gives for every neighbour question, and why
    /// [`SolidView::face_holds_the_point`] and
    /// [`SolidView::box_around_one_face`] hang here too. A [`FaceView`] does
    /// not know which solid it came from.
    ///
    /// # Two paths, the solid's own
    ///
    /// Whichever path [`SolidView::measure`] takes for this solid, this takes
    /// for the face: the face's own fan when every face of the solid is flat,
    /// exactly and with no mesh built, and otherwise the triangles of the
    /// solid's mesh that [`MeshTriangle::face`] says are this face's. Taking the
    /// solid's path rather than one of the face's own is what makes the faces
    /// add up to the solid bit for bit; a flat face with straight sides in a
    /// curved solid is therefore measured over the mesh and charged its rim,
    /// which is a bound it keeps rather than an exact zero it could have had.
    ///
    /// `deflection` and `policy` mean what they mean to [`SolidView::measure`].
    ///
    /// # Errors
    ///
    /// [`Error::NoSuchFaceInThisSolid`] when `face` names a face this solid
    /// does not use.
    ///
    /// [`Error::SurfaceAreaTooLargeToMeasure`] when the face's area is a real
    /// number `f64` has no room for — a face of a cube of side `1e200`. **A
    /// limit and not a gap** (§16.10): the answer exists and the type cannot
    /// hold it, the refusal [`SolidView::measure`] gives for the same reason.
    ///
    /// For a face of a curved solid, whatever [`SolidView::mesh`] refuses —
    /// [`Error::DeflectionBelowWhatAMeshCanReach`] among them — because the
    /// face is measured over the solid's mesh.
    ///
    /// Whatever [`Model::validate`](crate::Model::validate) refuses, when the
    /// face comes out with no area at all. A face with no area has no centroid
    /// — the answer does not exist, so that is a limit — and every face of a
    /// solid `validate` accepts has area: a flat one winds its outer wire
    /// certified positive round its normal, and a curved one's sides are each
    /// used by two triangles of the solid's mesh. So a face with none is a
    /// model `validate` refuses, and its refusal is the one handed back.
    ///
    /// # Panics
    ///
    /// For a face of a curved solid, everything [`SolidView::mesh`] panics on.
    /// And on a face with no area in a model
    /// [`Model::validate`](crate::Model::validate) accepts, which the paragraph
    /// above says cannot happen.
    pub fn measure_one_face(
        self,
        face: FaceKey,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<FaceMeasurement> {
        let view = self
            .faces()
            .find(|used| used.key() == face)
            .ok_or(Error::NoSuchFaceInThisSolid { face })?;
        let measured = if self.is_wholly_flat() {
            one_face_measured_over_its_own_fan(view)
        } else {
            let mesh = self.mesh(deflection, policy, log)?;
            self.one_face_measured_over(&mesh, face)
        };
        match measured {
            OneFace::Measured(measurement) => Ok(measurement),
            OneFace::TooLargeToMeasure => Err(Error::SurfaceAreaTooLargeToMeasure),
            OneFace::HasNoArea => {
                self.model().validate(policy, log)?;
                panic!(
                    "face {face:?} has no area in a model validate accepts, and every face \
                     of a sound solid has some"
                )
            }
        }
    }

    /// One face's share of [`SolidView::measured_over`]: the area
    /// [`SolidView::skin_of`] adds for it, the charge it adds for it, and the
    /// centroid of the same triangles.
    fn one_face_measured_over(self, mesh: &Mesh, face: FaceKey) -> OneFace {
        let triangles = the_triangles_of(mesh, face);
        let area = area_of(mesh, triangles);
        if !area.is_finite() {
            return OneFace::TooLargeToMeasure;
        }
        if area <= 0.0 {
            return OneFace::HasNoArea;
        }

        let bend = bend_of_each_face(self, mesh)[&face];
        let deviation = mesh.deviation().magnitude();
        let (area_error, moved, unpaired) = match (bend.tip_charge, bend.radius) {
            (Some(charge), _) => (
                charge.more_surface,
                deviation,
                3.0 * charge.more_surface + 2.0 * charge.below_the_split,
            ),
            (None, Some(radius)) => {
                let error = how_much_more_surface_there_can_be(area, radius, deviation);
                (error, deviation, 3.0 * error)
            }
            (None, None) => {
                let error = deviation * rim_of_the_flat_faces(mesh, |other| other == face);
                let plane = self
                    .faces()
                    .find(|used| used.key() == face)
                    .expect("the face was measured as one of this solid's")
                    .plane();
                let off = how_far_the_points_stray_off(mesh, triangles, &plane);
                (error, off, error)
            }
        };

        let centroid = middle_of(mesh, triangles);
        let reach = triangles
            .iter()
            .flat_map(|triangle| corners_of(mesh, triangle))
            .map(|corner| corner.distance_to(centroid))
            .fold(0.0, f64::max)
            + deviation;
        let centroid_wobble = if area_error < area {
            moved + reach * unpaired / (area - area_error)
        } else {
            f64::INFINITY
        };
        OneFace::Measured(FaceMeasurement {
            area,
            area_error,
            centroid,
            centroid_wobble,
        })
    }
}

/// What measuring one face came to, before it is told apart into an answer and
/// the refusals.
enum OneFace {
    Measured(FaceMeasurement),
    TooLargeToMeasure,
    HasNoArea,
}

/// One face of a wholly flat solid, over its own fan: the area
/// [`SolidView::surface_area`] adds for it, bit for bit, and the centroid of the
/// same triangles, both exact in the sense [`Measurement`] means.
///
/// **The fan's triangles are weighed with their sign**, which is what makes a
/// fan right for a face that is not convex or has a hole: the triangles over a
/// notch or a hole come back wound the other way, face against the face's own
/// direction, and take back the area and the moment they wrongly added.
fn one_face_measured_over_its_own_fan(face: FaceView<'_>) -> OneFace {
    let OwnFan {
        across,
        facing,
        twice,
        area,
        area_error,
    } = OwnFan::of(face);
    if !area.is_finite() {
        return OneFace::TooLargeToMeasure;
    }
    if area <= 0.0 {
        return OneFace::HasNoArea;
    }

    let mut apex = None;
    let mut moment = Vector::zero();
    let mut weight = 0.0;
    face.for_each_triangle(|[a, b, c]| {
        let along = shrink(b - a, across);
        let down = shrink(c - a, across);
        let share = along.cross(down).dot(facing);
        apex.get_or_insert(a);
        moment = moment + (along + down) * share;
        weight += share;
    });
    let apex = apex.expect("a face with area has a triangle");
    let centroid = apex + shrink(divided(moment, 3.0 * weight), -across);
    OneFace::Measured(FaceMeasurement {
        area,
        area_error,
        centroid,
        centroid_wobble: how_far_the_middle_of_its_fan_rounded(face, across, &twice, centroid),
    })
}

/// One face's own fan, measured: the power of two its sides were shrunk by,
/// the way it faces in those units, its vector area exactly, its area, and how
/// far that area can be from the area the fan really has.
///
/// The one copy of the arithmetic, so a face measured alone and the same face
/// summed into [`SolidView::surface_area`] are the same numbers bit for bit.
struct OwnFan {
    across: i32,
    facing: Vector,
    /// Twice the fan's vector area, exactly, in the units `across` picked: the
    /// sum over its triangles of `(b − a) × (c − a)`, one expansion per axis.
    twice: [Vec<f64>; 3],
    area: f64,
    area_error: f64,
}

impl OwnFan {
    fn of(face: FaceView<'_>) -> Self {
        let across = shift_for(largest_edge_component(&face));
        let facing = which_way_its_own_fan_faces(face, across);
        let length = facing.length();
        let twice = twice_its_vector_area_exactly(face, across);
        Self {
            across,
            facing,
            area: shift(length, 2 * across),
            area_error: shift_up(how_far_the_area_rounded(&twice, length), 2 * across),
            twice,
        }
    }
}

/// [`which_way_its_own_fan_faces`] doubled, before it rounded: each side of
/// each triangle taken exactly with [`two_diff`], and the cross products
/// multiplied out in [`rcad_predicates::expansion`], so nothing is lost.
fn twice_its_vector_area_exactly(face: FaceView<'_>, across: i32) -> [Vec<f64>; 3] {
    let mut twice: [Vec<f64>; 3] = Default::default();
    face.for_each_triangle(|[a, b, c]| {
        let turned = crossed(&exactly_along(a, b, across), &exactly_along(a, c, across));
        for (total, part) in twice.iter_mut().zip(&turned) {
            *total = packed(&added(total, part));
        }
    });
    twice
}

/// How far `length` — the rounded length of a face's vector area, in the units
/// `across` picked — is from the exact length of the same vector, rounded up.
///
/// **The truth is worked out, not bounded in advance.** `twice` is the fan's
/// vector area doubled with nothing lost, and `S = |twice|²` is exact too. The
/// area is `√S / 2`, which no `f64` need hold, so the distance is taken
/// without a square root:
///
/// ```text
///     |√S/2 − length| = |S − 4·length²| / (2·(√S + 2·length)) ≤ |S − 4·length²| / (4·length)
/// ```
///
/// and `S − 4·length²` is exact as well. A face whose rounded length came out
/// on the nose owes exactly nothing, which is every face with corners on whole
/// numbers.
///
/// A rounded length of nothing leaves nothing to divide by, and then the whole
/// of `√S / 2` is the distance.
fn how_far_the_area_rounded(twice: &[Vec<f64>; 3], length: f64) -> f64 {
    let squared = twice.iter().fold(Vec::new(), |total, part| {
        packed(&added(&total, &multiplied(part, part)))
    });
    if length == 0.0 {
        return divided_up(upward(at_most(&squared).sqrt()), 2.0);
    }
    let (four_squared, lost) = two_product(2.0 * length, 2.0 * length);
    let off = added(&squared, &[-lost, -four_squared]);
    divided_up(at_most(&off), 4.0 * length)
}

/// How far `middle` — where [`one_face_measured_over_its_own_fan`] put the
/// face's balance point — is from where the face's own fan balances exactly,
/// rounded up: a length, in the model's own unit.
///
/// # Which middle is the exact one
///
/// **The middle of the fan's triangles, each weighed by its signed shadow on
/// the fan's exact vector area.** Write `N` for `twice`, the sum over the
/// triangles of `(b − a) × (c − a)` with nothing lost. A triangle weighs
/// `((b − a) × (c − a)) · N` and stands at its own middle `(a + b + c) / 3`,
/// and the face's middle is the weighted average of those.
///
/// It needs choosing because a turned face's corners are not in one plane:
/// the turn rounded each of them on its own. **Where the corners are in one
/// plane, every sensible choice is the same point** — each cross product is
/// then parallel to `N`, so each weight is the triangle's area times one
/// common factor, with the sign it faces, and the weighted average is the
/// textbook middle of the face the corners bound, holes and notches taken
/// back. Off one plane the choices part, and the three worth naming are:
///
/// - **The shadow on the rounded vector area**, which is what the `f64` sum
///   weighs by. A truth made out of a rounded number moves whenever the
///   arithmetic that rounds it does, so it is not a truth about the face.
/// - **Each triangle's own area, with the sign it faces.** A square root per
///   triangle, so there is no exact sum for the rounding to be held against.
/// - **The shadow on the exact vector area**, this one. It is made of the
///   corners alone, by `+`, `−` and `×` and one division at the end, so it can
///   be worked out exactly; and its weights add up to `|N|²`, which is `|N|`
///   times the doubled area [`FaceMeasurement::area_error`] is measured
///   against — so a face's area and its middle are about the same shadow.
///
/// Those three weigh a triangle differently only through how far it leans off
/// the others times a second small angle — the cosine's own step from one, or
/// the tilt of the rounded vector area — so they part by a product of two
/// small angles, far below a rounding of the answer. **What does part them by
/// a rounding is the triangles themselves**: fanned from
/// another corner, the same corners off one plane make another surface. So
/// the fan [`FaceView::for_each_triangle`] hands out is part of the
/// definition, as it is of the area's and of the solid's volume's.
///
/// # How far, worked out
///
/// The weights and the weighted corners are summed exactly, and
/// [`how_far_the_middle_is_from`] holds `middle` against their ratio. A face
/// whose middle came out on the nose owes exactly nothing, which is every face
/// of a cube on whole numbers.
fn how_far_the_middle_of_its_fan_rounded(
    face: FaceView<'_>,
    across: i32,
    twice: &[Vec<f64>; 3],
    middle: Point,
) -> f64 {
    let mut apex = None;
    let mut moment: [Vec<f64>; 3] = Default::default();
    face.for_each_triangle(|[a, b, c]| {
        apex.get_or_insert(a);
        let along = exactly_along(a, b, across);
        let down = exactly_along(a, c, across);
        let turned = crossed(&along, &down);
        let shadow = packed(
            &turned
                .iter()
                .zip(twice)
                .fold(Vec::new(), |total, (part, of)| {
                    added(&total, &multiplied(part, of))
                }),
        );
        for (axis, total) in moment.iter_mut().enumerate() {
            let corners = added(&along[axis], &down[axis]);
            *total = packed(&added(total, &multiplied(&shadow, &corners)));
        }
    });
    let apex = apex.expect("a face with area has a triangle");
    let weights = twice.iter().fold(Vec::new(), |total, part| {
        added(&total, &multiplied(part, part))
    });
    let three_times_the_weights = packed(&multiplied(&weights, &[3.0]));
    how_far_the_middle_is_from(middle, apex, [across; 3], &moment, &three_times_the_weights)
}

/// `along × down`, exactly, for two sides [`exactly_along`] took.
fn crossed(along: &[[f64; 2]; 3], down: &[[f64; 2]; 3]) -> [Vec<f64>; 3] {
    [(1, 2), (2, 0), (0, 1)].map(|(second, third)| {
        let turning = multiplied(&along[second], &down[third]);
        let against = multiplied(&along[third], &down[second]);
        added(&turning, &negated(&against))
    })
}

/// How far `volume` is from a sixth of `six_volumes`, the fan's exact sum,
/// rounded up: the rounding of the sum, of the division by six, and of the
/// shift back to the model's own units, all three.
///
/// `sixth` is the volume still in the units the shifts picked. The exact six
/// volumes less six times it is exact in [`rcad_predicates::expansion`], and a
/// sixth of that is the distance there; [`shift_up`] carries it back without
/// rounding it down. Shifting `sixth` itself back is exact unless the volume
/// lands among the subnormals, and then it is off by less than their spacing,
/// which is charged.
///
/// **The difference is packed before its size is read**, for the reason
/// [`how_far_the_middle_is_from`] packs its own: fresh out of the subtraction
/// its entries can nearly cancel, and adding up their sizes then charges more
/// than the distance. The block less the cone in
/// `a_flat_solid_publishes_the_rounding_of_its_measure.rs` came out as
/// `7.95e-16` and `−1.78e-15` in the units the shifts picked, a distance of
/// `9.8e-16` charged as `2.57e-15`, and published `2.2e-13` for a mesh
/// `8.4e-14` off.
fn how_far_the_volume_rounded(
    six_volumes: &[f64],
    sixth: f64,
    volume: f64,
    whole_shift: i32,
) -> f64 {
    let (six_sixths, lost) = two_product(sixth, 6.0);
    let off = packed(&added(six_volumes, &[-lost, -six_sixths]));
    let error = shift_up(divided_up(at_most(&off), 6.0), whole_shift);
    if shift(volume, -whole_shift) == sixth {
        return error;
    }
    upward(error + f64::from_bits(1))
}

/// How far `middle` is from the middle a fan's exact sums stand for, rounded
/// up: a length, in the model's own unit.
///
/// Along each axis the exact middle is `from` plus `moment / weight`, a
/// ratio of two expansions in units `2^shifts[axis]` of the model's. Its
/// distance from `middle` is `|offset · weight − moment| / |weight|`, where
/// `offset` is `middle − from` taken exactly with [`two_diff`], so the top is
/// exact and only the division and the length of the three distances round,
/// both upwards. The top is packed before its size is read, because the
/// entries of an expansion fresh out of a subtraction can nearly cancel, and
/// adding up their sizes would then charge up to twice the distance.
/// **Worked out, not bounded in advance**, as the volume's and area's are: a
/// middle that came out on the nose owes exactly nothing.
///
/// Infinite when nothing keeps `weight` away from zero, because then there is
/// no middle to be near.
fn how_far_the_middle_is_from(
    middle: Point,
    from: Point,
    shifts: [i32; 3],
    moment: &[Vec<f64>; 3],
    weight: &[f64],
) -> f64 {
    let floor = at_least(weight);
    if floor == 0.0 {
        return f64::INFINITY;
    }
    let middle = middle.coordinates();
    let from = from.coordinates();
    length_at_most([0, 1, 2].map(|axis| {
        let (near, lost) = two_diff(middle[axis], from[axis]);
        let offset = [shift(lost, -shifts[axis]), shift(near, -shifts[axis])];
        let off = packed(&added(
            &multiplied(&offset, weight),
            &negated(&moment[axis]),
        ));
        shift_up(divided_up(at_most(&off), floor), shifts[axis])
    }))
}

/// How long a vector whose three sides are no longer than `sides` can be,
/// rounded up.
///
/// **The sides are scaled so the longest is between one and two before they
/// are squared, and the length is scaled back after.** A side under about
/// `1e-162` squares to below the smallest subnormal, and [`multiplied_up`] can
/// only step that up to `2^-1074`: three sides of `5e-301` came out `5.4e-162`
/// long, not `8.7e-301`. The longest side is read in the subnormals too, by
/// first lifting it out of them. Scaling by a power of two is exact, save for a
/// side so much shorter than the longest that it lands among the subnormals,
/// and [`shift_up`] rounds that one up, so every step still rounds up.
fn length_at_most(sides: [f64; 3]) -> f64 {
    let longest = sides.into_iter().fold(0.0, f64::max);
    let by = if longest.is_subnormal() {
        shift_for(shift(longest, 64)) - 64
    } else {
        shift_for(longest)
    };
    let squared = sides.into_iter().fold(0.0, |total, side| {
        let side = shift_up(side, -by);
        upward(total + multiplied_up(side, side))
    });
    shift_up(upward(squared.sqrt()), by)
}

/// `left × right` for two sizes, rounded up — off zero too, where two sizes
/// that are not zero multiply to less than the smallest subnormal.
fn multiplied_up(left: f64, right: f64) -> f64 {
    if left == 0.0 || right == 0.0 {
        return 0.0;
    }
    (left * right).next_up()
}

/// `b − a` along each axis, exactly, with both parts shrunk by `2^across`: the
/// side [`which_way_its_own_fan_faces`] rounds, before it rounded.
fn exactly_along(a: Point, b: Point, across: i32) -> [[f64; 2]; 3] {
    let from = a.coordinates();
    let to = b.coordinates();
    [0, 1, 2].map(|axis| {
        let (near, lost) = two_diff(to[axis], from[axis]);
        [shift(lost, -across), shift(near, -across)]
    })
}

/// Every triangle of a fan as nine exact offsets — three corners of three
/// axes, each the two `f64` [`exactly_how_far_from`] splits it into.
type ExactFan = Vec<[[f64; 2]; 9]>;

/// How far `corner` is from `hub` along each axis, exactly, in the units
/// `shifts` picked: the `f64` difference the rounded sum uses, and the bit of
/// that difference the subtraction threw away.
///
/// **The scaling is done to the difference and not to the two coordinates**,
/// because the shifts are read off the offsets and say nothing about where the
/// solid sits. A part `1e-300` across at `1e300` from the origin has a shift of
/// about `2^997`, which no coordinate of it survives being multiplied by and
/// every offset of it does.
///
/// The two entries are the expansion of the offset, smallest first, which is
/// the order everything in [`rcad_predicates::expansion`] wants.
fn exactly_how_far_from(hub: Point, corner: Point, shifts: [i32; 3]) -> [[f64; 2]; 3] {
    let hub = hub.coordinates();
    let corner = corner.coordinates();
    [0, 1, 2].map(|axis| {
        let (near, lost) = two_diff(corner[axis], hub[axis]);
        [shift(lost, -shifts[axis]), shift(near, -shifts[axis])]
    })
}

/// The offset the plain `f64` subtraction would have given, which is the larger
/// entry of each axis.
fn the_rounded_part_of(offset: [[f64; 2]; 3]) -> Vector {
    Vector::from(offset.map(|axis| axis[1]))
}

/// The three axes of an offset added up, ignoring their signs.
///
/// The product of this over the three corners of a triangle dominates the sum
/// of the six terms of their determinant, which is what a forward error bound
/// on that determinant is a multiple of.
fn across_the_axes(offset: Vector) -> f64 {
    offset
        .components()
        .iter()
        .map(|component| component.abs())
        .sum()
}

/// Six times the signed volume of a whole fan, exactly.
///
/// Each triangle is nine offsets — three corners of three axes — and each
/// offset is two `f64` that add up to it with nothing lost. From there to the
/// sum is nothing but `+`, `-` and `×` on those, and
/// [`rcad_predicates::expansion`] does all three without throwing a bit away,
/// so what comes back has the sign the solid really has.
///
/// The entries never overlap and come out smallest first, so adding them up in
/// that order lands on the nearest `f64` to the true sum: the last addition is
/// the only one that can round, and it rounds the whole answer once.
fn six_volumes_exactly(fan: &[[[f64; 2]; 9]]) -> f64 {
    let mut total: Vec<f64> = Vec::new();
    for triangle in fan {
        total = added(&total, &determinant_of(triangle));
    }
    total.iter().sum()
}

/// The same exact sum as [`six_volumes_exactly`], kept as an expansion, and
/// the first moment about the hub exactly too: the two sums
/// [`SolidView::volume_and_moment`] rounds, before they rounded.
///
/// The moment is each triangle's six volumes times the sum of its three
/// corners, so `hub + moment / (4 × six volumes)` is where the solid's own
/// triangles balance exactly — every tetrahedron they make with the hub
/// weighed by its signed volume and standing at its middle, a quarter of the
/// way from the hub to the sum of the corners. The triangles form a closed
/// surface, so that point is the same whatever the hub; which triangles they
/// are is part of it, as it is of the volume (`docs/decisions.md` §338.5).
///
/// **Packed after every triangle, because this runs on every solid measured,
/// over its fan or its mesh,** and not only on one whose sign is in doubt: an
/// expansion that is only ever added to grows by every entry of every
/// determinant, and adding to it costs its length each time. [`compress`]
/// keeps it as short as the value needs.
fn volume_and_moment_as_expansions(fan: &[[[f64; 2]; 9]]) -> (Vec<f64>, [Vec<f64>; 3]) {
    let mut six_volumes: Vec<f64> = Vec::new();
    let mut moment: [Vec<f64>; 3] = Default::default();
    for triangle in fan {
        let six_volume = determinant_of(triangle);
        six_volumes = packed(&added(&six_volumes, &six_volume));
        let six_volume = packed(&six_volume);
        for (axis, total) in moment.iter_mut().enumerate() {
            let corners = added(
                &added(&triangle[axis], &triangle[3 + axis]),
                &triangle[6 + axis],
            );
            *total = packed(&added(total, &multiplied(&six_volume, &corners)));
        }
    }
    (six_volumes, moment)
}

/// The exact determinant of one triangle's three offsets.
///
/// The six terms of a `3 × 3` determinant: three that take the axes in an even
/// order and count for themselves, three in an odd order and count against.
fn determinant_of(triangle: &[[f64; 2]; 9]) -> Vec<f64> {
    let mut total: Vec<f64> = Vec::new();
    for (axes, turn) in [
        ([0, 1, 2], 1.0),
        ([1, 2, 0], 1.0),
        ([2, 0, 1], 1.0),
        ([0, 2, 1], -1.0),
        ([2, 1, 0], -1.0),
        ([1, 0, 2], -1.0),
    ] {
        let pair = multiplied(&triangle[axes[0]], &triangle[3 + axes[1]]);
        let term = multiplied(&pair, &triangle[6 + axes[2]]);
        let signed: Vec<f64> = term.iter().map(|entry| turn * entry).collect();
        total = added(&total, &signed);
    }
    total
}

/// Two expansions multiplied out: every entry of one scaled by every entry of
/// the other, all added up.
fn multiplied(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut total: Vec<f64> = Vec::new();
    let mut piece = vec![0.0; 2 * left.len()];
    for &factor in right {
        let used = scale_expansion(left, factor, &mut piece);
        total = added(&total, &piece[..used]);
    }
    total
}

fn added(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut sum = vec![0.0; left.len() + right.len()];
    let used = expansion_sum(left, right, &mut sum);
    sum.truncate(used);
    sum
}

fn packed(expansion: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0; expansion.len()];
    let used = compress(expansion, &mut out);
    out.truncate(used);
    out
}

fn negated(expansion: &[f64]) -> Vec<f64> {
    expansion.iter().map(|entry| -entry).collect()
}

/// No less than the size of the value `expansion` stands for: every entry's
/// size added up, each addition rounded up.
fn at_most(expansion: &[f64]) -> f64 {
    expansion
        .iter()
        .fold(0.0, |total, entry| upward(total + entry.abs()))
}

/// No more than the size of the value `expansion` stands for: its last
/// entry's size less every other entry's, rounded down, and `0.0` where the
/// others could cancel it.
///
/// The last entry is the largest in every expansion
/// [`rcad_predicates::expansion`] hands back, and after [`compress`] it is
/// within a rounding of the whole value, so on a packed expansion this is
/// within a rounding or two of the size itself.
fn at_least(expansion: &[f64]) -> f64 {
    let Some((last, rest)) = expansion.split_last() else {
        return 0.0;
    };
    let rest = at_most(rest);
    if last.abs() <= rest {
        return 0.0;
    }
    (last.abs() - rest).next_down()
}

/// The next `f64` up from a sum of sizes, which may have rounded down, so a
/// bound built out of roundings is never smaller than what it stands for.
///
/// One step is enough: a correctly rounded sum is less than one spacing from
/// the exact one, and the next `f64` up is a whole spacing above it. **Zero
/// stays zero**, because a sum of sizes is zero only when every size was, and
/// then nothing rounded — which is what keeps a cube on whole numbers owing
/// exactly nothing.
fn upward(sum: f64) -> f64 {
    if sum == 0.0 { 0.0 } else { sum.next_up() }
}

/// `size / by`, rounded up.
///
/// Not [`upward`] of the quotient: a size that is not zero can be divided all
/// the way down to zero among the subnormals, and that zero would stay. This
/// steps off it whenever `size` is not zero.
fn divided_up(size: f64, by: f64) -> f64 {
    if size == 0.0 {
        return 0.0;
    }
    (size / by).next_up()
}

/// The widest component of any edge of any of `face`'s triangles.
///
/// The face's own extent, not its distance from the hub. A small face a long
/// way out has small edges, and scaling it by how far away it is would flush
/// its area to nothing.
fn largest_edge_component(face: &FaceView<'_>) -> f64 {
    let mut widest = 0.0f64;
    face.for_each_triangle(|[a, b, c]| {
        for edge in [b - a, c - a] {
            for component in edge.components() {
                widest = widest.max(component.abs());
            }
        }
    });
    widest
}

/// Half the sum of the cross products of `face`'s fan, with every side shrunk by
/// `2^across`: a vector as long as the face's area in those units, pointing the
/// way the face faces.
///
/// The one copy of the arithmetic, so a face measured alone and the same face
/// summed into [`SolidView::surface_area`] are the same number bit for bit.
fn which_way_its_own_fan_faces(face: FaceView<'_>, across: i32) -> Vector {
    let mut vector_area = Vector::zero();
    face.for_each_triangle(|[a, b, c]| {
        let along = shrink(b - a, across);
        let down = shrink(c - a, across);
        vector_area = vector_area + along.cross(down) * 0.5;
    });
    vector_area
}

/// How much skin a mesh has, and how much of its solid's own it leaves out.
struct Skin {
    area: f64,
    area_error: f64,
    volume_error: f64,
}

/// The corner the sums over a mesh hang from, picked exactly as
/// [`SolidView::hub`] picks one: the smallest, `x` first.
fn hub_of(mesh: &Mesh) -> Point {
    let mut smallest: Option<[f64; 3]> = None;
    for point in mesh.points() {
        let corner = point.coordinates();
        if smallest.is_none_or(|so_far| lexicographically_smaller(corner, so_far)) {
            smallest = Some(corner);
        }
    }
    Point::from(smallest.unwrap_or([0.0; 3]))
}

/// One power of two per axis over a mesh's points, for
/// [`SolidView::axis_shifts`]'s reasons.
fn axis_shifts_over(mesh: &Mesh, hub: Point) -> [i32; 3] {
    let mut widest = [0.0f64; 3];
    for point in mesh.points() {
        let offsets = (*point - hub).components();
        for (axis, so_far) in widest.iter_mut().enumerate() {
            *so_far = so_far.max(offsets[axis].abs());
        }
    }
    widest.map(shift_for)
}

/// Six times the signed volume of a mesh, its first moment about `hub`, how far
/// the first can be from the six volumes the mesh's triangles enclose exactly,
/// and every triangle's offsets exactly.
///
/// The mesh's triangles in the order it stores them, which is fixed (§25.7), so
/// two runs give the same sum bit for bit.
///
/// The sign is settled the same way [`SolidView::volume_and_moment`] settles
/// it, and for the same reason: `Model::validate` calls a negative one
/// [`Invalid::VolumeIsNotPositive`], and the fuse's `nest_the_shells` sorts a
/// curved shell into a lump or a void of one by it. What the exact sum here is
/// exact *about* is the mesh, which stands in for the solid to within
/// [`Measurement::volume_error`]; the sign it decides is the sign the mesh's
/// own triangles have, which is what these two callers branch on.
///
/// The rounded sum is off by at most its slack, which is why it is only kept
/// when it is further from zero than that. The exact sum is the nearest `f64`
/// to the true one, off by half a spacing of itself, which `|sum| · ε` plus
/// the smallest subnormal covers. An overflow comes back with a slack that is
/// not finite, so it is off by any amount, which is what it is.
///
/// The offsets are what [`SolidView::measured_over`] works out how far its
/// rounded volume landed from, as [`SolidView::volume_and_moment`]'s are for a
/// flat solid.
fn volume_and_moment_over(mesh: &Mesh, hub: Point, shifts: [i32; 3]) -> SumsOverAMesh {
    let mut fan: Vec<[[f64; 2]; 9]> = Vec::new();
    let mut six_volumes = 0.0;
    let mut how_big_the_terms_are = 0.0;
    let mut moment = [0.0f64; 3];
    for triangle in mesh.triangles() {
        let exact =
            corners_of(mesh, triangle).map(|corner| exactly_how_far_from(hub, corner, shifts));
        let [a, b, c] = exact.map(the_rounded_part_of);
        let six_volume = a.dot(b.cross(c));
        six_volumes += six_volume;
        how_big_the_terms_are += across_the_axes(a) * across_the_axes(b) * across_the_axes(c);
        let offsets = [a, b, c].map(|offset| offset.components());
        for (axis, total) in moment.iter_mut().enumerate() {
            *total += six_volume * (offsets[0][axis] + offsets[1][axis] + offsets[2][axis]);
        }
        fan.push(core::array::from_fn(|slot| exact[slot / 3][slot % 3]));
    }

    let slack = (fan.len() as f64 + 8.0) * f64::EPSILON * how_big_the_terms_are;
    if six_volumes.abs() > slack || !how_big_the_terms_are.is_finite() {
        return SumsOverAMesh {
            six_volumes,
            moment,
            six_volumes_off: upward(slack),
            fan,
        };
    }
    let exactly = six_volumes_exactly(&fan);
    SumsOverAMesh {
        six_volumes: exactly,
        moment,
        six_volumes_off: exactly.abs() * f64::EPSILON + f64::from_bits(1),
        fan,
    }
}

/// What [`volume_and_moment_over`] sums, in the units the shifts picked.
struct SumsOverAMesh {
    six_volumes: f64,
    moment: [f64; 3],
    /// How far `six_volumes` can be from the exact sum, rounded up: a bound
    /// ahead of time, where [`how_far_the_volume_rounded`] works it out.
    six_volumes_off: f64,
    fan: ExactFan,
}

/// The signed volume of a mesh, for
/// [`Model::validate`](crate::Model::validate) to complain about — the curved
/// twin of [`SolidView::signed_volume`].
pub(crate) fn signed_volume_over(mesh: &Mesh) -> f64 {
    let hub = hub_of(mesh);
    let shifts = axis_shifts_over(mesh, hub);
    let sums = volume_and_moment_over(mesh, hub, shifts);
    shift(sums.six_volumes / 6.0, shifts.iter().sum())
}

/// [`signed_volume_over`], and how far it can be from the volume the mesh's
/// own triangles enclose exactly, rounded up.
///
/// **The sign is always the mesh's own; the size need not be close.** The
/// rounded sum stands whenever it is further from zero than its slack, so a
/// sum just past its slack can be off by nearly all of itself. A caller that
/// compares the size with something else needs this second number, and
/// `validate`'s check of a mesh that lies flat on one plane is one.
///
/// It is the six volumes' own error, a sixth of it, the rounding of that
/// sixth, and of the shift back to the model's units, all rounded up, the way
/// [`how_far_the_volume_rounded`] charges a fan's. The six volumes' error is
/// one of two bounds this file already rests the sign on: the slack
/// [`SolidView::volume_and_moment`] derives, for a rounded sum, and half a
/// spacing, for the exact one [`six_volumes_exactly`] rounds once.
pub(crate) fn signed_volume_over_and_how_far_it_rounded(mesh: &Mesh) -> (f64, f64) {
    let hub = hub_of(mesh);
    let shifts = axis_shifts_over(mesh, hub);
    let sums = volume_and_moment_over(mesh, hub, shifts);
    let whole_shift = shifts.iter().sum();
    let sixth = sums.six_volumes / 6.0;
    let volume = shift(sixth, whole_shift);
    let sixth_off = upward(
        divided_up(sums.six_volumes_off, 6.0) + sixth.abs() * f64::EPSILON + f64::from_bits(1),
    );
    let error = shift_up(sixth_off, whole_shift);
    if shift(volume, -whole_shift) == sixth {
        return (volume, error);
    }
    (volume, upward(error + f64::from_bits(1)))
}

/// Each face of the mesh and the triangles that stand in for it, in the order
/// the mesh stores them.
///
/// A face's triangles arrive together, so this is a walk and not a lookup.
fn each_face_and_its_triangles(mesh: &Mesh) -> Vec<(FaceKey, &[MeshTriangle])> {
    let triangles = mesh.triangles();
    let mut faces = Vec::new();
    let mut start = 0;
    while start < triangles.len() {
        let face = triangles[start].face();
        let mut end = start;
        while end < triangles.len() && triangles[end].face() == face {
            end += 1;
        }
        faces.push((face, &triangles[start..end]));
        start = end;
    }
    faces
}

/// The area of one face's triangles, measured in a scale of its own for the
/// reason [`SolidView::surface_area`] gives.
///
/// **Every triangle's area added, not the length of their vector sum.** A flat
/// face's triangle normals all point the same way and the two agree there; a
/// cylinder's wall goes right round and its vector areas cancel to nothing, so
/// the shorter sum would say a bar has no wall.
fn area_of(mesh: &Mesh, triangles: &[MeshTriangle]) -> f64 {
    let mut widest = 0.0f64;
    for triangle in triangles {
        let [a, b, c] = corners_of(mesh, triangle);
        for edge in [b - a, c - a] {
            for component in edge.components() {
                widest = widest.max(component.abs());
            }
        }
    }
    let across = shift_for(widest);

    let mut total = 0.0;
    for triangle in triangles {
        let [a, b, c] = corners_of(mesh, triangle);
        let along = shrink(b - a, across);
        let down = shrink(c - a, across);
        total += along.cross(down).length() * 0.5;
    }
    shift(total, 2 * across)
}

/// The triangles of `mesh` that stand in for `face`, which the mesh keeps
/// together; empty when the face has none.
///
/// # Panics
///
/// If the face's triangles are not all in one run. The mesher cuts up one face
/// at a time, and [`each_face_and_its_triangles`] — what the solid's area is
/// summed from — reads a face twice over if they are not, so the two would
/// disagree.
fn the_triangles_of(mesh: &Mesh, face: FaceKey) -> &[MeshTriangle] {
    let triangles = mesh.triangles();
    let start = triangles
        .iter()
        .position(|triangle| triangle.face() == face)
        .unwrap_or(triangles.len());
    let end = start
        + triangles[start..]
            .iter()
            .take_while(|triangle| triangle.face() == face)
            .count();
    assert!(
        triangles[end..]
            .iter()
            .all(|triangle| triangle.face() != face),
        "the triangles of face {face:?} are not in one run"
    );
    &triangles[start..end]
}

/// Where the triangles balance, each weighed by its area.
///
/// Hung from the first corner and measured in one power of two, for the
/// reasons [`SolidView::volume_and_moment`] and [`area_of`] give: offsets the
/// size of the face rather than of its distance from the origin, and no
/// product that can overflow on the way.
///
/// **Unsigned areas**, as [`area_of`] adds them: a mesh's triangles do not
/// overlap and all face out of the material, so there is nothing to cancel.
fn middle_of(mesh: &Mesh, triangles: &[MeshTriangle]) -> Point {
    let hub = corners_of(mesh, &triangles[0])[0];
    let mut widest = 0.0f64;
    for triangle in triangles {
        for corner in corners_of(mesh, triangle) {
            for component in (corner - hub).components() {
                widest = widest.max(component.abs());
            }
        }
    }
    let across = shift_for(widest);

    let mut moment = Vector::zero();
    let mut weight = 0.0;
    for triangle in triangles {
        let [a, b, c] = corners_of(mesh, triangle).map(|corner| shrink(corner - hub, across));
        let share = (b - a).cross(c - a).length();
        moment = moment + (a + b + c) * share;
        weight += share;
    }
    hub + shrink(divided(moment, 3.0 * weight), -across)
}

/// `moment` with every component divided by `weight`, each rounding once.
///
/// **The third of a triangle's corners is taken here, once, and not per
/// triangle**, so a face whose corners sit on whole numbers — a cube's — has
/// every product and sum before this exact, and its middle comes out the
/// nearest `f64` to the true one rather than a few roundings off it.
fn divided(moment: Vector, weight: f64) -> Vector {
    Vector::from(moment.components().map(|component| component / weight))
}

/// How much more surface a curved face can have than the `area` of the flat
/// triangles standing in for it.
///
/// **The derivation, for a face on a sphere of radius `r` cut to a deviation of
/// `d`.** Every point of every triangle is on the near side of the surface and
/// no further than `d` from it, so all of them sit between the radii `r − d` and
/// `r` — which is all the argument uses, so it covers a hole's wall on the same
/// lines as a ball's, with the material on the other side of it. Projecting the triangles radially in onto the sphere of radius `r − d`
/// scales an area element by `cos θ · ((r − d) / ρ)²`, where `ρ ≥ r − d` is how
/// far out the element is and `θ` is how far its normal leans off the radial
/// direction — never more than one — so the solid angle the face covers is at
/// most `area / (r − d)²`. The true surface over that solid angle has area
/// `r²` times it, so it is at most `(r / (r − d))²` of `area`, and this is the
/// difference.
///
/// **A cylinder is covered by the same number and is not derived twice.** The
/// same argument about the axis gives `r / (r − d)`, which is the smaller of the
/// two for every `r > d`, so the sphere's factor bounds both. One expression
/// that is loose for pipes beats two that can disagree.
///
/// Infinite when the mesh may be a whole radius away from the surface: there is
/// then no shell to trap the difference in and nothing here bounds it. A
/// deflection that coarse is a legitimate request (§23.5) and this is what it
/// costs.
fn how_much_more_surface_there_can_be(area: f64, radius: f64, deviation: f64) -> f64 {
    let inner = radius - deviation;
    if inner <= 0.0 {
        return f64::INFINITY;
    }
    area * ((radius / inner) * (radius / inner) - 1.0)
}

/// How much space lies between a curved face's triangles and the surface they
/// stand in for, over the `area` of those triangles.
///
/// How much, not which way: on a bar that space is material the mesh left out
/// and round a hole it is material the mesh put in, and
/// [`Measurement::volume_error`] is where that is said.
///
/// The same shell as [`how_much_more_surface_there_can_be`], now measured rather
/// than projected. The face covers a solid angle of at most `area / (r − d)²`,
/// and the shell between the radii `r − d` and `r` over that angle holds
/// `(r³ − (r − d)³) / 3` per unit of it.
///
/// **None of this is charged to a flat face**: it does not bend, so no shell
/// lies between its triangles and its surface. What a flat face owes is how
/// far its mesh points stray off its plane, which is
/// [`how_far_the_points_stray_off`] and [`SolidView::skin_of`] charges it. The
/// sliver over a flat face's *rim* — between its inscribed outline and the
/// curve that outline stands for — is inside the neighbouring curved face's
/// shell and already counted — which the short wide bar in
/// `the_published_error_bounds_cover_what_a_mesh_of_a_bar_or_a_ball_misses`
/// checks against `πr²h`.
///
/// Infinite for a mesh a whole radius out, for the reason above.
fn how_much_space_hides_under_a_flat_stand_in(area: f64, radius: f64, deviation: f64) -> f64 {
    let inner = radius - deviation;
    if inner <= 0.0 {
        return f64::INFINITY;
    }
    area * (radius * radius * radius - inner * inner * inner) / (3.0 * inner * inner)
}

/// How far the furthest corner of `triangles` lies off `plane`, rounded up: a
/// length, in the model's own unit.
///
/// **Worked out exactly and not read off [`Plane::signed_distance_to`]**,
/// whose [`Plane::SIGNED_DISTANCE_ROUNDOFF`] at coordinates of `4` is about
/// `3.6e-15`, eight times the `4.3e-16` the sliver's caps really stray. Each
/// product of the normal and a coordinate is split into its rounded part and
/// what rounding lost, so the sum less the offset is the exact `n · p − o`.
/// That is the distance times the normal's length, and a turned normal is
/// one only to within [`Plane::NORMAL_LENGTH_ROUNDOFF`] of its square, so
/// dividing by the length is at most multiplying by `1 + 64ε`.
///
/// `0.0` exactly when every corner sits on the plane exactly.
fn how_far_the_points_stray_off(mesh: &Mesh, triangles: &[MeshTriangle], plane: &Plane) -> f64 {
    let normal = plane.normal().components();
    let mut furthest = 0.0f64;
    for triangle in triangles {
        for corner in corners_of(mesh, triangle) {
            let mut height = vec![-plane.offset()];
            for (along, at) in normal.into_iter().zip(corner.coordinates()) {
                let (product, lost) = two_product(along, at);
                height = added(&height, &[lost, product]);
            }
            furthest = furthest.max(at_most(&packed(&height)));
        }
    }
    upward(furthest * (1.0 + Plane::NORMAL_LENGTH_ROUNDOFF))
}

/// The total length of the mesh sides where a face `counted` picks meets another
/// face: every flat face, for the solid, and the one face being measured, for
/// [`FaceMeasurement`].
///
/// **A flat face of a curved solid is not exact either, and this is why.** A
/// round bar's cap is a disc, and what the mesh draws is a polygon inscribed in
/// its rim — short of the disc by one circular segment per side. Each segment
/// fits inside a rectangle as long as its chord and as tall as the deviation, so
/// `deviation` times this length bounds the lot.
///
/// Counted once for each flat face along a side, which is what makes a bar's two
/// caps cost twice one rim. A side used twice by the *same* face is a seam and
/// is not a rim at all, so it is skipped by construction.
fn rim_of_the_flat_faces(mesh: &Mesh, counted: impl Fn(FaceKey) -> bool) -> f64 {
    let mut first_seen: HashMap<(u32, u32), FaceKey> = HashMap::new();
    let mut rim = 0.0;
    for triangle in mesh.triangles() {
        let [first, second, third] = triangle.corners();
        for (from, to) in [(first, second), (second, third), (third, first)] {
            let side = (from.min(to), from.max(to));
            let Some(&already) = first_seen.get(&side) else {
                first_seen.insert(side, triangle.face());
                continue;
            };
            if already == triangle.face() {
                continue;
            }
            let length = mesh.points()[from as usize].distance_to(mesh.points()[to as usize]);
            for face in [already, triangle.face()] {
                if counted(face) {
                    rim += length;
                }
            }
        }
    }
    rim
}

/// How far out of `centroid` any part of the solid can be: the furthest mesh
/// point, and the deviation on top, because the surface stands off the mesh by
/// that much again.
fn how_far_the_solid_reaches_from(mesh: &Mesh, centroid: Point) -> f64 {
    let furthest = mesh
        .points()
        .iter()
        .map(|point| point.distance_to(centroid))
        .fold(0.0, f64::max);
    furthest + mesh.deviation().magnitude()
}

/// How far each face of this solid bends, or `None` where it does not bend at
/// all.
///
/// # How small a cone face's radius really gets, and why the corners cannot say
///
/// **Read over the mesh's own points and not over the face's corners**, and the
/// case that forces it is a cone that ends in a point. Its cone face is bounded
/// by one circle and nothing else; the apex is where the surface's own second
/// coordinate runs out, not a vertex of the model, so **no corner of that face
/// is anywhere near it**. Charged against its corners the face would publish the
/// radius at its rim — four units on a cone of radius four — for a face that
/// reaches a radius of nothing at all. That is a bound claimed and not earned,
/// which is worse than no bound.
///
/// The mesh's points are the right domain anyway: what the two bounds above are
/// about is how far *these triangles* can be from the surface, and the triangles
/// are what the mesh holds.
///
/// **The offset-surface argument is what needs the smallest and not the
/// largest.** [`how_much_more_surface_there_can_be`] projects the mesh onto the
/// surface `deviation` inside the material and reads the area back out; for a
/// cone that inner surface is *the same cone shifted along its own axis*, so the
/// factor is the pipe's `R/(R − d)` with `R = slope·|v|·√(1 + slope²)` the radius
/// of normal curvature — and the sphere's `(R/(R − d))²` bounds that, exactly as
/// it bounds a pipe's. `R` varies over the face, so the smallest of it is the
/// one that bounds the lot.
///
/// **A face that reaches its apex gets zero, and on its own that makes both
/// bounds infinite.** Within `deviation` of the apex there is no inner offset
/// cone at all — its own apex has moved `deviation·√(1 + slope²)/slope` up the
/// axis — so no argument of this shape covers the tip. [`what_a_cone_tip_costs`]
/// is what covers it, and a face that reaches in there comes back with its whole
/// charge already worked out rather than a radius to work one out from — because
/// that charge is a sum over the face's own triangles and not a factor on its
/// area. Every other face is left exactly as it was.
///
/// # Panics
///
/// On a surface this has not been taught. [`Surface`] grows a variant at every
/// milestone, and a surface measured by not being measured is worse than a stop.
fn bend_of_each_face(solid: SolidView<'_>, mesh: &Mesh) -> HashMap<FaceKey, Bend> {
    let mut cones: HashMap<FaceKey, Cone> = HashMap::new();
    let mut bends: HashMap<FaceKey, Bend> = solid
        .faces()
        .map(|face| {
            let radius = match face.surface() {
                Surface::Plane(_) => None,
                Surface::Sphere(ball) => Some(ball.radius()),
                Surface::Cylinder(pipe) => Some(pipe.radius()),
                // Worked out below, over the mesh's own points rather than over
                // the face's corners. A face with no triangles keeps the
                // infinity, and contributes no area either, so it is charged
                // nothing — which is right, because there is nothing there.
                Surface::Cone(cone) => {
                    cones.insert(face.key(), cone);
                    Some(f64::INFINITY)
                }
                // **The tube radius, and it needs no mesh to find.** The inner
                // offset surface of a torus is the same torus with a thinner
                // tube, which exists for every `d` below `r` — so unlike a
                // cone, a torus never runs out of an inner surface to project
                // onto, and unlike a ball its ratio is not one number. The area
                // element is `(R + r·cos v)·r`, so the offset's is
                // `(R + (r − d)cos v)(r − d)` and the ratio is
                // `[r/(r − d)]·[(R + r·cos v)/(R + (r − d)cos v)]`. The second
                // factor is largest at the outer equator, where it is
                // `(R + r)/(R + r − d)` — smaller than `r/(r − d)` because
                // `R + r` is bigger than `r` — so the whole ratio is under
                // `(r/(r − d))²`, which is the sphere's factor with `r` in it.
                Surface::Torus(ring) => Some(ring.tube_radius()),
                other => panic!("this crate cannot say how far {other:?} bends"),
            };
            (
                face.key(),
                Bend {
                    radius,
                    tip_charge: None,
                },
            )
        })
        .collect();

    for triangle in mesh.triangles() {
        let Some(cone) = cones.get(&triangle.face()) else {
            continue;
        };
        let along = cone.axis().direction();
        let steepness = cone.slope().hypot(1.0);
        for corner in triangle.corners() {
            let at = mesh.points()[corner as usize];
            let here = cone.radius_at(along.dot(at - cone.apex())) * steepness;
            let slot = bends
                .get_mut(&triangle.face())
                .expect("every face of the solid is in the table");
            slot.radius = Some(
                slot.radius
                    .expect("a cone face was put in with a radius")
                    .min(here),
            );
        }
    }

    let deviation = mesh.deviation().magnitude();
    for (face, cone) in cones {
        let split = 2.0 * deviation * (1.0 + cone.slope() * cone.slope());
        let slot = bends
            .get_mut(&face)
            .expect("every face of the solid is in the table");
        if slot.radius.expect("a cone face was put in with a radius") < split {
            slot.tip_charge = Some(what_a_cone_tip_costs(mesh, face, &cone, deviation));
        }
    }
    bends
}

/// How far one face bends, and what it costs where that is not a question a
/// radius can answer.
#[derive(Clone, Copy)]
struct Bend {
    /// The smallest radius of normal curvature the face reaches, or `None`
    /// where the face does not bend at all and is exact.
    radius: Option<f64>,
    /// What this face costs outright, on a cone face that reaches inside its
    /// own tip. `None` on every other face, which is charged off `radius` as
    /// it always was.
    tip_charge: Option<Charge>,
}

/// What one face costs the two published bounds.
#[derive(Clone, Copy)]
struct Charge {
    more_surface: f64,
    hidden_space: f64,
    /// All the cone's side below the split plus the triangles' share of it:
    /// the area near the apex that [`FaceMeasurement::centroid_wobble`] cannot
    /// pair with anything, because no inner surface exists down there to pair
    /// it through.
    below_the_split: f64,
}

/// What a cone face that reaches near its own apex costs, summed over its own
/// triangles.
///
/// **Where the split goes, and why there.** The inner offset cone — the same
/// cone slid `deviation·√(1 + slope²)/slope` up its own axis — is what
/// [`how_much_more_surface_there_can_be`] projects onto, and it does not exist
/// below its own apex. Splitting at `v = 2·deviation·√(1 + slope²)/slope`
/// leaves everything above it with a radius of normal curvature
/// `slope·v·√(1 + slope²) ≥ 2·deviation·(1 + slope²)`, clear of the deviation
/// by more than the deviation itself. That radius is what the split is written
/// in terms of here, because that is what the triangles are measured against.
///
/// **The split runs through the triangles and not between them.** A pointed
/// cone is meshed as a fan from its apex — a cone is developable, so every
/// edge from the apex to the rim lies exactly on the surface — and **every one
/// of those triangles has a corner at the apex**. Sorting whole triangles above
/// and below the split would put the entire face below it, and charging the
/// entire face at the split radius is a factor that does not depend on the
/// deviation at all: a bound that never shrinks.
///
/// **So the face is cut into bands of radius, each doubling on the last**, and
/// each band is charged off the radius at its own floor. The radius of normal
/// curvature is a fixed multiple of the height up the axis, which runs linearly
/// across a flat triangle, so what a band takes from one triangle is
/// [`the_share_of_a_triangle_below`] a height — a ratio of areas that needs no
/// clipping. A band's area grows like its radius squared while the factor
/// charged to it falls like one over that radius, so the sum is dominated by the
/// outermost band and comes out `O(deviation)`, which is the same order the
/// bound on a ball already has. Ten or so bands cover a cone of any size that
/// `f64` holds.
///
/// **What the tip below the split can hold.** A cone of height `v` has
/// `π·slope·v²·√(1 + slope²)` of side and `π·slope²·v³/3` of volume; at the
/// split those are `4π·deviation²(1 + slope²)^{3/2}/slope` and
/// `(8π/3)·deviation³(1 + slope²)^{3/2}/slope`. This face's own share of the tip
/// is no bigger than the whole of it, and the triangles standing in for that
/// share are chords of it, so the space between the two skins down there is
/// inside the same piece — whatever part of the tip the face covers, and however
/// it is meshed.
///
/// **The tip is charged the cheaper of its whole size and the plain bound**, so
/// no face is charged more than it was before this existed: a truncated cone
/// whose near cap stops just short of the split still has a real radius, and
/// taking the tip's whole size there would loosen a bound already earned. Where
/// the face does reach the apex the plain bound is infinite and the whole size
/// wins by default.
///
/// **A slope of nothing makes the tip infinite, and that is the truth** — a cone
/// that flat has a tip stretching away for ever. `Cone::new` refuses a slope of
/// zero, so nobody can build one; a slope small enough to make the tip bigger
/// than the plain bound is charged the plain bound by the line above.
fn what_a_cone_tip_costs(mesh: &Mesh, face: FaceKey, cone: &Cone, deviation: f64) -> Charge {
    let steepness = cone.slope().hypot(1.0);
    let along = cone.axis().direction();
    let mut triangles = Vec::new();
    let mut widest = 0.0f64;
    let mut smallest = f64::INFINITY;
    for (at, triangle) in mesh.triangles().iter().enumerate() {
        if triangle.face() != face {
            continue;
        }
        let mut radii = triangle.corners().map(|corner| {
            let point = mesh.points()[corner as usize];
            cone.radius_at(along.dot(point - cone.apex())) * steepness
        });
        radii.sort_by(f64::total_cmp);
        widest = widest.max(radii[2]);
        smallest = smallest.min(radii[0]);
        triangles.push((radii, area_of(mesh, &mesh.triangles()[at..=at])));
    }

    let area_below = |level: f64| -> f64 {
        triangles
            .iter()
            .map(|(radii, area)| area * the_share_of_a_triangle_below(radii, level))
            .sum()
    };

    let split = 2.0 * deviation * steepness * steepness;
    let mut charge = the_whole_tip_below(cone, deviation);
    let under_the_split = area_below(split);
    charge.below_the_split += under_the_split;
    charge.more_surface = charge.more_surface.min(how_much_more_surface_there_can_be(
        under_the_split,
        smallest,
        deviation,
    ));
    charge.hidden_space = charge
        .hidden_space
        .min(how_much_space_hides_under_a_flat_stand_in(
            under_the_split,
            smallest,
            deviation,
        ));

    let mut floor = split;
    let mut taken = under_the_split;
    while floor < widest {
        floor *= 2.0;
        let reached = area_below(floor);
        let band = reached - taken;
        taken = reached;
        charge.more_surface += how_much_more_surface_there_can_be(band, floor * 0.5, deviation);
        charge.hidden_space +=
            how_much_space_hides_under_a_flat_stand_in(band, floor * 0.5, deviation);
    }
    charge
}

/// All the side and all the space a cone can hold below the split, which is
/// what a face reaching its apex is charged for the part of itself down there.
fn the_whole_tip_below(cone: &Cone, deviation: f64) -> Charge {
    let pi = core::f64::consts::PI;
    let steepness = cone.slope().hypot(1.0);
    let spread = steepness * steepness * steepness / cone.slope().abs();
    let side = 4.0 * pi * deviation * deviation * spread;
    Charge {
        more_surface: side,
        hidden_space: 8.0 * pi * deviation * deviation * deviation * spread / 3.0,
        below_the_split: side,
    }
}

/// How much of a triangle's area lies below a `level`, as a share of the whole,
/// where each corner is at the height its entry in `radii` says and the height
/// runs linearly between them.
///
/// `radii` is sorted. A plane below the lowest corner cuts off nothing and one
/// above the highest cuts off everything; in between it cuts off a triangle
/// similar to the one the lowest corner is in, or leaves one similar to the one
/// the highest corner is in, and a similar triangle's share is the product of
/// the two ratios along the sides it is cut from.
fn the_share_of_a_triangle_below(radii: &[f64; 3], level: f64) -> f64 {
    let [low, middle, high] = *radii;
    if level <= low {
        return 0.0;
    }
    if level >= high {
        return 1.0;
    }
    if level <= middle {
        return (level - low) * (level - low) / ((middle - low) * (high - low));
    }
    1.0 - (high - level) * (high - level) / ((high - middle) * (high - low))
}

/// Where one mesh triangle's three corners are.
fn corners_of(mesh: &Mesh, triangle: &MeshTriangle) -> [Point; 3] {
    triangle.corners().map(|at| mesh.points()[at as usize])
}

/// The exponent that brings `value` to between one and two, or `0` when there
/// is no exponent to read: `value` is zero, subnormal, or not a number.
///
/// A solid whose extent along an axis is subnormal is §19.6's low-end limit and
/// scaling does not reach it.
fn shift_for(value: f64) -> i32 {
    if !value.is_normal() {
        return 0;
    }
    i32::try_from(value.to_bits() >> 52 & 0x7ff).expect("eleven bits") - 1023
}

/// `offset` with every axis shrunk by the same power of two.
fn shrink(offset: Vector, by: i32) -> Vector {
    let components = offset.components();
    Vector::from(components.map(|component| shift(component, -by)))
}

/// `value * 2^exponent`, in steps small enough to stay in range.
///
/// **In steps, because `2^exponent` need not be representable when the answer
/// is.** A slab `2^350` across and `2^300` thick encloses `2^1000`, which fits,
/// while the `2^1050` that would scale it back in one multiplication does not.
/// Every step here moves the same way, so the running value walks towards the
/// answer and never past it, which leaves overflow meaning what it should:
/// `f64` has no room for the number.
fn shift(value: f64, exponent: i32) -> f64 {
    let mut shifted = value;
    let mut left = exponent;
    while left != 0 {
        let step = left.clamp(-512, 512);
        shifted *= two_to_the(step);
        left -= step;
    }
    shifted
}

/// [`shift`] for a bound, which must not come out smaller than it went in.
///
/// Shifting up is exact until it overflows, and infinity is no smaller than
/// anything. Shifting down is exact until it reaches the subnormals, where it
/// can round down; shifting the answer back up, which is exact, is what says
/// whether it did.
fn shift_up(size: f64, exponent: i32) -> f64 {
    let shifted = shift(size, exponent);
    if shift(shifted, -exponent) < size {
        return shifted.next_up();
    }
    shifted
}

/// `2^exponent`, exactly, for an exponent an `f64` can hold.
fn two_to_the(exponent: i32) -> f64 {
    f64::from_bits(u64::try_from(1023 + exponent).expect("a normal exponent") << 52)
}

/// Whether `corner` sorts before `so_far`, comparing `x`, then `y`, then `z`.
fn lexicographically_smaller(corner: [f64; 3], so_far: [f64; 3]) -> bool {
    corner
        .iter()
        .zip(&so_far)
        .find_map(|(mine, theirs)| match mine.total_cmp(theirs) {
            core::cmp::Ordering::Equal => None,
            core::cmp::Ordering::Less => Some(true),
            core::cmp::Ordering::Greater => Some(false),
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Isometry, Scale, Vector};

    use crate::{Model, ProfileBuilder, Rule, SolidKey};

    use super::*;

    const PI: f64 = core::f64::consts::PI;

    /// A plate `side` across and `thick` deep with a hole of radius `radius`
    /// bored through the middle, and the skin it really has.
    ///
    /// Two rectangles and a bar rather than a builder that takes a hole,
    /// because the fuse is what puts a curved wall inside a flat face.
    fn a_plate_with_a_hole(side: f64, thick: f64, radius: f64) -> (Model, SolidKey, f64) {
        let policy = Policy::certified();
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let rectangle =
            ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
                .build(&policy, &mut ProvenanceLog::new())
                .expect("a square");
        let plate = model
            .add_extrusion(&rectangle, thick, &Isometry::identity())
            .expect("a plate");
        let bar = model
            .add_round_bar(
                radius,
                thick + 4.0,
                &Isometry::translation(Vector::new(side / 2.0, side / 2.0, -2.0)).expect("a shift"),
            )
            .expect("a bar longer than the plate is thick");
        let holed = model
            .fuse(
                &[plate],
                &[bar],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a plate with a hole through it");
        let holed = holed.solids()[0];

        let true_area = 2.0 * (side * side - PI * radius * radius)
            + 4.0 * side * thick
            + 2.0 * PI * radius * thick;
        (model, holed, true_area)
    }

    /// [`Measurement::area_error`] is a distance and not a shortfall, and this
    /// is the pair of shapes that says it has to be.
    ///
    /// A meshed hole gets its two ends and its wall wrong in opposite
    /// directions. The flat face's outline round the hole is a polygon
    /// **inscribed** in the circle, so it takes less away than the circle does
    /// and that face comes out too large; the wall's chords are shorter than the
    /// arcs they stand for, so the wall comes out too small. Which one wins is
    /// how deep the hole is, so the same kind of solid lands on both sides of
    /// the answer and only the magnitude can be published.
    ///
    /// Both true areas are closed forms worked out here from what the shapes
    /// were built with, not read back off the kernel.
    #[test]
    fn a_thin_plate_with_a_hole_measures_more_skin_than_it_has() {
        let policy = Policy::certified();
        let measured = |model: &Model, solid: SolidKey, how_fine: f64| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(how_fine).expect("a length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a plate with a hole measures")
        };

        for how_fine in [0.1, 0.03, 0.01] {
            let (model, plate, true_area) = a_plate_with_a_hole(8.0, 0.5, 1.0);
            let thin = measured(&model, plate, how_fine);
            let over = thin.surface_area() - true_area;
            println!("a plate half a unit thick at {how_fine}: {over:+} of skin too much");
            assert!(
                over > 0.0,
                "a hole through a thin plate has to measure more skin than the {true_area} it \
                 has, and it measured {}",
                thin.surface_area()
            );

            let (model, block, true_area) = a_plate_with_a_hole(8.0, 8.0, 1.0);
            let deep = measured(&model, block, how_fine);
            let under = deep.surface_area() - true_area;
            println!("a hole eight deep at {how_fine}: {under:+} of skin");
            assert!(
                under < 0.0,
                "a hole eight units deep has to measure less skin than the {true_area} it has, \
                 and it measured {}",
                deep.surface_area()
            );

            // Which is the whole point: the published number covers both signs,
            // and neither of the two would be covered by a one-sided one.
            for (out_by, measurement) in [(over, thin), (under, deep)] {
                assert!(
                    measurement.area_error().is_finite(),
                    "nothing is worth asserting against an infinite bound (§47.4)"
                );
                assert!(
                    out_by.abs() <= measurement.area_error(),
                    "the skin is {out_by} out and the kernel owned up to {}",
                    measurement.area_error()
                );
            }
        }
    }

    /// The volume goes the other way round on the same two shapes, which is what
    /// makes [`Measurement::volume_error`] a distance too.
    ///
    /// `a_block_with_a_round_hole_measures_bigger_than_it_is` in
    /// `what_a_deflection_costs_a_measured_volume.rs` is the same finding
    /// against a closed-form volume; this is the half that says the two error
    /// terms do not agree on a sign even for one solid, so neither of them can
    /// publish one.
    #[test]
    fn a_hole_makes_the_volume_read_high_where_the_same_hole_makes_the_skin_read_low() {
        let policy = Policy::certified();
        let (model, block, _) = a_plate_with_a_hole(8.0, 8.0, 1.0);
        let true_volume = (64.0 - PI) * 8.0;

        for how_fine in [0.1, 0.03, 0.01] {
            let measurement = model
                .solid(block)
                .expect("this model made the key")
                .measure(
                    Deflection::of(how_fine).expect("a length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("the block measures");
            let over = measurement.volume() - true_volume;
            println!("a hole eight deep at {how_fine}: {over:+} of material too much");
            assert!(
                over > 0.0,
                "the mesh of a hole is a prism inside it, so the block has to come out bigger \
                 than the {true_volume} it is, and it came out {}",
                measurement.volume()
            );
            assert!(
                over <= measurement.volume_error(),
                "the block is {over} over and the kernel owned up to {}",
                measurement.volume_error()
            );
        }
    }
}
