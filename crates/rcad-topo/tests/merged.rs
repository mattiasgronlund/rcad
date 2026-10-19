//! Most of rcad-topo's integration tests, built as one test binary.
//!
//! Every file directly under `tests/` used to be a binary of its own, and each
//! one linked the whole kernel again. Every worktree writes all of its test
//! binaries, since kache restores each one as a private copy, and those bytes
//! are what is wearing out the SSD (`docs/decisions.md` §373). Declared here as
//! modules, these files link the kernel once.
//!
//! Each file is declared with `#[path]` and stays where it is. A file reached
//! through `#[path]` looks for its own `mod cones;` at `tests/cones/mod.rs`,
//! just as it did as a crate root, so no test file had to change and every line
//! the record cites is still on that line. `pub mod` keeps a file's `pub` items
//! as live as they were when the file was a crate root.
//!
//! A file the justfile or the record names by binary stays a binary of its own,
//! with a `[[test]]` in `Cargo.toml`, so that command still works.
//!
//! `autotests` is off, so a new file in `tests/` is built by nothing until it is
//! declared here, and `every_file_in_tests_is_built_into_exactly_one_test_binary`
//! fails until then, printing the two lines to add. They go in the file's place
//! by name, and the lane that adds the file adds them. No build script writes
//! them, since one would rebuild the kernel and every crate above it at each
//! edit of a test (`docs/decisions.md` §373.7).

// A helper such as `tests/cones/mod.rs` is a module of every file that declares
// it, so this binary holds one copy of it per file, as the separate binaries did.
#![allow(clippy::duplicate_mod)]

use std::collections::BTreeMap;
use std::path::Path;

// `tests/pairs/mod.rs` reads `crate::cones`, and the crate root is this file now.
mod cones;

#[path = "a_ball_and_a_bar_in_one_block.rs"]
pub mod a_ball_and_a_bar_in_one_block;
#[path = "a_ball_at_a_turned_cubes_corner_capped_at_both_poles.rs"]
pub mod a_ball_at_a_turned_cubes_corner_capped_at_both_poles;
#[path = "a_ball_centred_a_hair_off_a_cubes_face.rs"]
pub mod a_ball_centred_a_hair_off_a_cubes_face;
#[path = "a_ball_clear_of_a_block_comes_back_whole.rs"]
pub mod a_ball_clear_of_a_block_comes_back_whole;
#[path = "a_ball_cut_through_both_poles_by_a_linked_blocks_plane.rs"]
pub mod a_ball_cut_through_both_poles_by_a_linked_blocks_plane;
#[path = "a_ball_cut_through_its_equator_meshes_each_half_on_its_own_side.rs"]
pub mod a_ball_cut_through_its_equator_meshes_each_half_on_its_own_side;
#[path = "a_ball_face_holding_both_poles_inside_three_arcs.rs"]
pub mod a_ball_face_holding_both_poles_inside_three_arcs;
#[path = "a_ball_face_ringing_its_pole_round_a_round_hole.rs"]
pub mod a_ball_face_ringing_its_pole_round_a_round_hole;
#[path = "a_ball_face_with_several_holes.rs"]
pub mod a_ball_face_with_several_holes;
#[path = "a_ball_hollowed_by_a_turned_cube_keeps_its_skin_whole.rs"]
pub mod a_ball_hollowed_by_a_turned_cube_keeps_its_skin_whole;
#[path = "a_ball_hollowed_by_an_off_centre_bar_keeps_its_skin_whole.rs"]
pub mod a_ball_hollowed_by_an_off_centre_bar_keeps_its_skin_whole;
#[path = "a_ball_inscribed_in_a_cube.rs"]
pub mod a_ball_inscribed_in_a_cube;
#[path = "a_ball_joined_to_a_block_down_its_polar_axis.rs"]
pub mod a_ball_joined_to_a_block_down_its_polar_axis;
#[path = "a_ball_kept_by_a_block_and_a_leaning_plane.rs"]
pub mod a_ball_kept_by_a_block_and_a_leaning_plane;
#[path = "a_ball_kept_to_a_corner_of_a_box.rs"]
pub mod a_ball_kept_to_a_corner_of_a_box;
#[path = "a_ball_less_a_cap_whose_rim_runs_through_its_north_pole.rs"]
pub mod a_ball_less_a_cap_whose_rim_runs_through_its_north_pole;
#[path = "a_ball_poking_through_a_cubes_side.rs"]
pub mod a_ball_poking_through_a_cubes_side;
#[path = "a_ball_split_on_itself_united_with_a_block_clear_of_it.rs"]
pub mod a_ball_split_on_itself_united_with_a_block_clear_of_it;
#[path = "a_ball_touching_a_prisms_edge_exactly.rs"]
pub mod a_ball_touching_a_prisms_edge_exactly;
#[path = "a_ball_touching_the_edge_between_two_tilted_walls.rs"]
pub mod a_ball_touching_the_edge_between_two_tilted_walls;
#[path = "a_ball_whose_seam_is_a_little_past_a_quarter_turn.rs"]
pub mod a_ball_whose_seam_is_a_little_past_a_quarter_turn;
#[path = "a_bar_and_a_ball_on_its_cap_turned.rs"]
pub mod a_bar_and_a_ball_on_its_cap_turned;
#[path = "a_bar_cut_beside_its_axis_and_united_again.rs"]
pub mod a_bar_cut_beside_its_axis_and_united_again;
#[path = "a_bar_flush_inside_a_turned_blocks_side.rs"]
pub mod a_bar_flush_inside_a_turned_blocks_side;
#[path = "a_bar_inside_a_tubes_wall.rs"]
pub mod a_bar_inside_a_tubes_wall;
#[path = "a_bar_lying_along_a_plane_or_a_bar.rs"]
pub mod a_bar_lying_along_a_plane_or_a_bar;
#[path = "a_bar_moved_by_a_tiny_step_fuses_as_the_unmoved_bar.rs"]
pub mod a_bar_moved_by_a_tiny_step_fuses_as_the_unmoved_bar;
#[path = "a_bar_moved_by_whole_numbers_keeps_its_seam_corner_exact.rs"]
pub mod a_bar_moved_by_whole_numbers_keeps_its_seam_corner_exact;
#[path = "a_bar_notched_by_a_block.rs"]
pub mod a_bar_notched_by_a_block;
#[path = "a_bar_on_a_tube_touching_along_one_circle.rs"]
pub mod a_bar_on_a_tube_touching_along_one_circle;
#[path = "a_bar_turned_three_quarter_turns_and_halved.rs"]
pub mod a_bar_turned_three_quarter_turns_and_halved;
#[path = "a_bar_whose_wall_is_two_faces.rs"]
pub mod a_bar_whose_wall_is_two_faces;
#[path = "a_bend_round_a_slanted_axis.rs"]
pub mod a_bend_round_a_slanted_axis;
#[path = "a_bend_under_two_planes.rs"]
pub mod a_bend_under_two_planes;
#[path = "a_bends_cap_holds_one_of_the_two_disks.rs"]
pub mod a_bends_cap_holds_one_of_the_two_disks;
#[path = "a_bent_edge_between_two_flat_faces_is_refused.rs"]
pub mod a_bent_edge_between_two_flat_faces_is_refused;
#[path = "a_block_cornered_on_a_round_tabs_rim.rs"]
pub mod a_block_cornered_on_a_round_tabs_rim;
#[path = "a_block_edge_grazing_a_ball.rs"]
pub mod a_block_edge_grazing_a_ball;
#[path = "a_block_edge_grazing_a_ball_at_cx_0.rs"]
pub mod a_block_edge_grazing_a_ball_at_cx_0;
#[path = "a_block_moved_by_whole_numbers_keeps_its_corners_exact.rs"]
pub mod a_block_moved_by_whole_numbers_keeps_its_corners_exact;
#[path = "a_block_poking_into_a_ball_whose_top_plane_cuts_the_rest.rs"]
pub mod a_block_poking_into_a_ball_whose_top_plane_cuts_the_rest;
#[path = "a_block_with_a_whole_ring_through_its_middle.rs"]
pub mod a_block_with_a_whole_ring_through_its_middle;
#[path = "a_bore_cut_across_by_a_second_fuse.rs"]
pub mod a_bore_cut_across_by_a_second_fuse;
#[path = "a_box_holds_every_curved_face.rs"]
pub mod a_box_holds_every_curved_face;
#[path = "a_box_inside_a_bar.rs"]
pub mod a_box_inside_a_bar;
#[path = "a_box_inside_a_frustum.rs"]
pub mod a_box_inside_a_frustum;
#[path = "a_cap_trimmed_by_a_pipe_holds_its_points.rs"]
pub mod a_cap_trimmed_by_a_pipe_holds_its_points;
#[path = "a_cap_whose_pole_no_partner_separates.rs"]
pub mod a_cap_whose_pole_no_partner_separates;
#[path = "a_cap_whose_rim_bends_back_past_its_own_meridian.rs"]
pub mod a_cap_whose_rim_bends_back_past_its_own_meridian;
#[path = "a_chamfered_bar_cut_by_a_block_turned_another_way.rs"]
pub mod a_chamfered_bar_cut_by_a_block_turned_another_way;
#[path = "a_closed_edge_cornered_a_hair_off_where_its_loop_was_walked_from.rs"]
pub mod a_closed_edge_cornered_a_hair_off_where_its_loop_was_walked_from;
#[path = "a_cone_and_its_copy.rs"]
pub mod a_cone_and_its_copy;
#[path = "a_cone_arc_past_the_far_end_of_a_long_ellipse.rs"]
pub mod a_cone_arc_past_the_far_end_of_a_long_ellipse;
#[path = "a_cone_cap_whose_rim_runs_through_its_apex.rs"]
pub mod a_cone_cap_whose_rim_runs_through_its_apex;
#[path = "a_cone_face_against_its_own_reversed_spelling.rs"]
pub mod a_cone_face_against_its_own_reversed_spelling;
#[path = "a_cone_hollowed_along_its_axis.rs"]
pub mod a_cone_hollowed_along_its_axis;
#[path = "a_cone_skin_beside_a_plane_its_open_curve_never_reaches.rs"]
pub mod a_cone_skin_beside_a_plane_its_open_curve_never_reaches;
#[path = "a_cone_whose_apex_sits_exactly_on_the_origin.rs"]
pub mod a_cone_whose_apex_sits_exactly_on_the_origin;
#[path = "a_corner_on_the_lattice_against_a_torus_face.rs"]
pub mod a_corner_on_the_lattice_against_a_torus_face;
#[path = "a_cube_set_on_another_cubes_corner_turned_about_a_leaning_axis.rs"]
pub mod a_cube_set_on_another_cubes_corner_turned_about_a_leaning_axis;
#[path = "a_face_between_two_tools_facing_opposite_ways.rs"]
pub mod a_face_between_two_tools_facing_opposite_ways;
#[path = "a_face_bounded_by_two_half_circles.rs"]
pub mod a_face_bounded_by_two_half_circles;
#[path = "a_face_bounded_by_two_steps.rs"]
pub mod a_face_bounded_by_two_steps;
#[path = "a_face_flush_against_another_solids_face.rs"]
pub mod a_face_flush_against_another_solids_face;
#[path = "a_flat_cap_cut_by_a_cones_rim.rs"]
pub mod a_flat_cap_cut_by_a_cones_rim;
#[path = "a_flat_face_with_an_arc_holds_a_box_on_a_diagonal.rs"]
pub mod a_flat_face_with_an_arc_holds_a_box_on_a_diagonal;
#[path = "a_flat_solid_whose_volume_signs_the_wrong_way.rs"]
pub mod a_flat_solid_whose_volume_signs_the_wrong_way;
#[path = "a_floor_holding_a_circle_cut_in_two.rs"]
pub mod a_floor_holding_a_circle_cut_in_two;
#[path = "a_fold_between_two_curves_closer_than_any_two_corners.rs"]
pub mod a_fold_between_two_curves_closer_than_any_two_corners;
#[path = "a_fourth_kind_of_run.rs"]
pub mod a_fourth_kind_of_run;
#[path = "a_frustum_and_a_block_that_do_not_touch.rs"]
pub mod a_frustum_and_a_block_that_do_not_touch;
#[path = "a_frustum_cut_in_half_down_its_axis.rs"]
pub mod a_frustum_cut_in_half_down_its_axis;
#[path = "a_frustum_sharpened_to_a_speck.rs"]
pub mod a_frustum_sharpened_to_a_speck;
#[path = "a_frustums_wall_cut_below_a_blocks_floor.rs"]
pub mod a_frustums_wall_cut_below_a_blocks_floor;
#[path = "a_fuse_is_not_blamed_for_a_solid_it_never_touched.rs"]
pub mod a_fuse_is_not_blamed_for_a_solid_it_never_touched;
#[path = "a_fuse_that_builds_a_sliver.rs"]
pub mod a_fuse_that_builds_a_sliver;
#[path = "a_half_ball_cut_again_away_from_its_poles.rs"]
pub mod a_half_ball_cut_again_away_from_its_poles;
#[path = "a_half_ball_holed_beside_its_pole_and_capped_on_the_same_ball.rs"]
pub mod a_half_ball_holed_beside_its_pole_and_capped_on_the_same_ball;
#[path = "a_half_ball_holed_round_its_pole_and_capped_on_the_same_ball.rs"]
pub mod a_half_ball_holed_round_its_pole_and_capped_on_the_same_ball;
#[path = "a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball.rs"]
pub mod a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball;
#[path = "a_half_ball_on_a_notched_plate.rs"]
pub mod a_half_ball_on_a_notched_plate;
#[path = "a_half_ball_on_a_pipe_meshes_whichever_face_is_listed_first.rs"]
pub mod a_half_ball_on_a_pipe_meshes_whichever_face_is_listed_first;
#[path = "a_half_ball_skin_with_a_round_hole.rs"]
pub mod a_half_ball_skin_with_a_round_hole;
#[path = "a_half_bar_turned_about_a_half_bends_axis.rs"]
pub mod a_half_bar_turned_about_a_half_bends_axis;
#[path = "a_half_ring_beside_a_bar_clear_of_it.rs"]
pub mod a_half_ring_beside_a_bar_clear_of_it;
#[path = "a_half_ring_built_face_by_face.rs"]
pub mod a_half_ring_built_face_by_face;
#[path = "a_half_ring_standing_on_a_round_plate.rs"]
pub mod a_half_ring_standing_on_a_round_plate;
#[path = "a_half_rings_flat_face_read_back_from_bytes.rs"]
pub mod a_half_rings_flat_face_read_back_from_bytes;
#[path = "a_holed_plate_and_a_block_on_it_turned.rs"]
pub mod a_holed_plate_and_a_block_on_it_turned;
#[path = "a_joined_flat_face_whose_rings_enclose_no_area.rs"]
pub mod a_joined_flat_face_whose_rings_enclose_no_area;
#[path = "a_key_from_another_model.rs"]
pub mod a_key_from_another_model;
#[path = "a_lens_whose_hole_is_half_a_disk.rs"]
pub mod a_lens_whose_hole_is_half_a_disk;
#[path = "a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.rs"]
pub mod a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches;
#[path = "a_mesh_refusal_names_the_face_that_stops_it.rs"]
pub mod a_mesh_refusal_names_the_face_that_stops_it;
#[path = "a_model_only_the_policy_that_built_it_calls_valid.rs"]
pub mod a_model_only_the_policy_that_built_it_calls_valid;
#[path = "a_model_that_declares_its_own_tolerance.rs"]
pub mod a_model_that_declares_its_own_tolerance;
#[path = "a_notched_block_less_a_box_names_its_refusal.rs"]
pub mod a_notched_block_less_a_box_names_its_refusal;
#[path = "a_piece_grazed_by_a_trimming_nothing_follows.rs"]
pub mod a_piece_grazed_by_a_trimming_nothing_follows;
#[path = "a_piece_no_ray_can_place_names_its_refusal.rs"]
pub mod a_piece_no_ray_can_place_names_its_refusal;
#[path = "a_piece_of_a_face_bounded_by_two_straight_sides.rs"]
pub mod a_piece_of_a_face_bounded_by_two_straight_sides;
#[path = "a_pin_beside_a_ring_it_never_touches.rs"]
pub mod a_pin_beside_a_ring_it_never_touches;
#[path = "a_pin_header_body_less_a_box.rs"]
pub mod a_pin_header_body_less_a_box;
#[path = "a_placed_cones_apex_and_the_point_the_mesh_puts_there.rs"]
pub mod a_placed_cones_apex_and_the_point_the_mesh_puts_there;
#[path = "a_placed_solid_hands_out_its_runs_where_it_is.rs"]
pub mod a_placed_solid_hands_out_its_runs_where_it_is;
#[path = "a_plane_across_a_cone.rs"]
pub mod a_plane_across_a_cone;
#[path = "a_plane_face_cut_by_a_cone_it_crosses_in_an_ellipse.rs"]
pub mod a_plane_face_cut_by_a_cone_it_crosses_in_an_ellipse;
#[path = "a_plane_is_never_one_surface_with_a_curved_face.rs"]
pub mod a_plane_is_never_one_surface_with_a_curved_face;
#[path = "a_plate_and_its_round_corner_tab_unite_at_every_turn.rs"]
pub mod a_plate_and_its_round_corner_tab_unite_at_every_turn;
#[path = "a_plate_cut_by_a_whole_bend.rs"]
pub mod a_plate_cut_by_a_whole_bend;
#[path = "a_plate_with_a_hole_beside_a_cut.rs"]
pub mod a_plate_with_a_hole_beside_a_cut;
#[path = "a_plate_with_an_off_centre_hole.rs"]
pub mod a_plate_with_an_off_centre_hole;
#[path = "a_point_a_rounding_past_a_turned_solids_corner.rs"]
pub mod a_point_a_rounding_past_a_turned_solids_corner;
#[path = "a_point_beside_a_turned_flat_face.rs"]
pub mod a_point_beside_a_turned_flat_face;
#[path = "a_point_in_one_faces_sliver_by_its_edge.rs"]
pub mod a_point_in_one_faces_sliver_by_its_edge;
#[path = "a_point_on_a_face_beside_a_trimming_nothing_follows.rs"]
pub mod a_point_on_a_face_beside_a_trimming_nothing_follows;
#[path = "a_pointed_cone_drilled_along_its_axis.rs"]
pub mod a_pointed_cone_drilled_along_its_axis;
#[path = "a_pole_corner_met_while_a_pole_side_is_open.rs"]
pub mod a_pole_corner_met_while_a_pole_side_is_open;
#[path = "a_pole_off_the_pinch_is_refused.rs"]
pub mod a_pole_off_the_pinch_is_refused;
#[path = "a_pole_on_a_cone_must_stand_at_its_apex.rs"]
pub mod a_pole_on_a_cone_must_stand_at_its_apex;
#[path = "a_prism_lying_on_one_edge.rs"]
pub mod a_prism_lying_on_one_edge;
#[path = "a_quarter_of_a_ball_built_face_by_face.rs"]
pub mod a_quarter_of_a_ball_built_face_by_face;
#[path = "a_refusal_that_names_the_wrong_corner.rs"]
pub mod a_refusal_that_names_the_wrong_corner;
#[path = "a_rim_a_fuse_cut_round_one_cone.rs"]
pub mod a_rim_a_fuse_cut_round_one_cone;
#[path = "a_rim_that_names_its_cutting_surface.rs"]
pub mod a_rim_that_names_its_cutting_surface;
#[path = "a_rim_walked_against_the_walk_read_back_from_bytes.rs"]
pub mod a_rim_walked_against_the_walk_read_back_from_bytes;
#[path = "a_ring_band_used_the_wrong_way_round_crosses_its_own_solid.rs"]
pub mod a_ring_band_used_the_wrong_way_round_crosses_its_own_solid;
#[path = "a_ring_face_bounded_by_a_circle_on_a_coaxial_surface.rs"]
pub mod a_ring_face_bounded_by_a_circle_on_a_coaxial_surface;
#[path = "a_ring_round_an_apex_on_a_face_that_also_has_a_hole.rs"]
pub mod a_ring_round_an_apex_on_a_face_that_also_has_a_hole;
#[path = "a_round_bar_notched_by_a_block_cornered_on_its_wall.rs"]
pub mod a_round_bar_notched_by_a_block_cornered_on_its_wall;
#[path = "a_rung_whose_ray_meets_a_face_nobody_follows.rs"]
pub mod a_rung_whose_ray_meets_a_face_nobody_follows;
#[path = "a_shell_inside_another_names_its_refusal.rs"]
pub mod a_shell_inside_another_names_its_refusal;
#[path = "a_sixth_kind_of_surface.rs"]
pub mod a_sixth_kind_of_surface;
#[path = "a_skin_that_closes_only_with_another_never_reads_back.rs"]
pub mod a_skin_that_closes_only_with_another_never_reads_back;
#[path = "a_sliver_along_an_edge_of_a_face_it_lies_on.rs"]
pub mod a_sliver_along_an_edge_of_a_face_it_lies_on;
#[path = "a_sliver_arc_on_a_joined_flat_face.rs"]
pub mod a_sliver_arc_on_a_joined_flat_face;
#[path = "a_sliver_on_the_plane_of_a_face_it_misses.rs"]
pub mod a_sliver_on_the_plane_of_a_face_it_misses;
#[path = "a_small_cap_cut_off_a_ball.rs"]
pub mod a_small_cap_cut_off_a_ball;
#[path = "a_solid_a_long_way_from_the_origin.rs"]
pub mod a_solid_a_long_way_from_the_origin;
#[path = "a_solid_carrying_a_run_along_a_cones_ruling.rs"]
pub mod a_solid_carrying_a_run_along_a_cones_ruling;
#[path = "a_solid_wholly_inside_another_comes_back_uncut.rs"]
pub mod a_solid_wholly_inside_another_comes_back_uncut;
#[path = "a_solid_with_a_cone_face.rs"]
pub mod a_solid_with_a_cone_face;
#[path = "a_solid_with_a_curved_face.rs"]
pub mod a_solid_with_a_curved_face;
#[path = "a_solid_with_a_torus_face.rs"]
pub mod a_solid_with_a_torus_face;
#[path = "a_stepped_shaft_cut_in_half_and_turned.rs"]
pub mod a_stepped_shaft_cut_in_half_and_turned;
#[path = "a_third_spelling_of_a_surface_a_face_never_sees.rs"]
pub mod a_third_spelling_of_a_surface_a_face_never_sees;
#[path = "a_tilted_wall_grazing_a_ball_sums_to_zero.rs"]
pub mod a_tilted_wall_grazing_a_ball_sums_to_zero;
#[path = "a_tube_less_a_bar_united_with_a_second_bar.rs"]
pub mod a_tube_less_a_bar_united_with_a_second_bar;
#[path = "a_turned_and_shifted_cut_along_a_seam_misses_it_by_a_rounding.rs"]
pub mod a_turned_and_shifted_cut_along_a_seam_misses_it_by_a_rounding;
#[path = "a_turned_ball_and_block_closes_an_edge_at_a_pole.rs"]
pub mod a_turned_ball_and_block_closes_an_edge_at_a_pole;
#[path = "a_turned_ball_and_block_that_would_not_measure.rs"]
pub mod a_turned_ball_and_block_that_would_not_measure;
#[path = "a_turned_bar_cut_by_a_block_turned_with_it.rs"]
pub mod a_turned_bar_cut_by_a_block_turned_with_it;
#[path = "a_turned_bar_whose_union_had_a_corner_under_the_triangulators_floor.rs"]
pub mod a_turned_bar_whose_union_had_a_corner_under_the_triangulators_floor;
#[path = "a_turned_block_and_a_ball_intersected_either_way_round.rs"]
pub mod a_turned_block_and_a_ball_intersected_either_way_round;
#[path = "a_turned_block_whose_face_holds_a_balls_pole.rs"]
pub mod a_turned_block_whose_face_holds_a_balls_pole;
#[path = "a_turned_frame_and_a_block_flush_beneath_it.rs"]
pub mod a_turned_frame_and_a_block_flush_beneath_it;
#[path = "a_turned_piece_lying_on_another_solids_face.rs"]
pub mod a_turned_piece_lying_on_another_solids_face;
#[path = "a_turned_plane_exactly_proportional_to_another.rs"]
pub mod a_turned_plane_exactly_proportional_to_another;
#[path = "a_turned_tooth_corner_on_the_edge_of_an_ell.rs"]
pub mod a_turned_tooth_corner_on_the_edge_of_an_ell;
#[path = "a_walk_neither_winding_nor_the_axis_places.rs"]
pub mod a_walk_neither_winding_nor_the_axis_places;
#[path = "a_walk_that_came_back_with_no_points.rs"]
pub mod a_walk_that_came_back_with_no_points;
#[path = "a_wire_that_rings_a_cones_apex.rs"]
pub mod a_wire_that_rings_a_cones_apex;
#[path = "an_edge_between_two_planes_its_curve_leaves.rs"]
pub mod an_edge_between_two_planes_its_curve_leaves;
#[path = "an_edge_with_one_face_along_it.rs"]
pub mod an_edge_with_one_face_along_it;
#[path = "an_ellipse_start_between_two_places.rs"]
pub mod an_ellipse_start_between_two_places;
#[path = "an_inside_out_solid_meshed_coarse_answers_negative.rs"]
pub mod an_inside_out_solid_meshed_coarse_answers_negative;
#[path = "an_open_run_ended_a_hair_off_its_crossing_a_thousand_from_the_origin.rs"]
pub mod an_open_run_ended_a_hair_off_its_crossing_a_thousand_from_the_origin;
#[path = "area_error_ignores_a_tilted_triangles_extra_area.rs"]
pub mod area_error_ignores_a_tilted_triangles_extra_area;
#[path = "building_solids.rs"]
pub mod building_solids;
#[path = "degenerate_curved_fuses.rs"]
pub mod degenerate_curved_fuses;
#[path = "degenerate_fuses.rs"]
pub mod degenerate_fuses;
#[path = "edge_history_either_way_round.rs"]
pub mod edge_history_either_way_round;
#[path = "every_model_the_fuse_fuzzer_kept_is_fused_again.rs"]
pub mod every_model_the_fuse_fuzzer_kept_is_fused_again;
#[path = "fuses_mean_what_their_rule_says.rs"]
pub mod fuses_mean_what_their_rule_says;
#[path = "how_far_a_balls_mesh_points_really_sit_from_the_ball.rs"]
pub mod how_far_a_balls_mesh_points_really_sit_from_the_ball;
#[path = "how_far_a_balls_mesh_strays_when_the_ball_sits_far_from_the_origin.rs"]
pub mod how_far_a_balls_mesh_strays_when_the_ball_sits_far_from_the_origin;
#[path = "how_far_apart_the_two_floors_under_a_mesh_are.rs"]
pub mod how_far_apart_the_two_floors_under_a_mesh_are;
#[path = "how_far_inside_a_triangle_a_mesh_really_strays.rs"]
pub mod how_far_inside_a_triangle_a_mesh_really_strays;
#[path = "how_long_an_edge_is.rs"]
pub mod how_long_an_edge_is;
#[path = "how_the_sag_of_a_balls_mesh_steps_with_the_deflection.rs"]
pub mod how_the_sag_of_a_balls_mesh_steps_with_the_deflection;
#[path = "measuring.rs"]
pub mod measuring;
#[path = "measuring_one_face.rs"]
pub mod measuring_one_face;
#[path = "meshing_a_curved_solid.rs"]
pub mod meshing_a_curved_solid;
#[path = "no_sequence_of_builder_calls_panics.rs"]
pub mod no_sequence_of_builder_calls_panics;
#[path = "one_pipe_whose_origin_sits_at_two_places.rs"]
pub mod one_pipe_whose_origin_sits_at_two_places;
#[path = "one_solid_named_twice_in_a_fuse.rs"]
pub mod one_solid_named_twice_in_a_fuse;
#[path = "ordering_points_along_an_endless_bend.rs"]
pub mod ordering_points_along_an_endless_bend;
#[path = "pieces_of_one_curved_surface.rs"]
pub mod pieces_of_one_curved_surface;
#[path = "point_in_a_curved_solid.rs"]
pub mod point_in_a_curved_solid;
#[path = "point_in_solid.rs"]
pub mod point_in_solid;
#[path = "random_extrusions.rs"]
pub mod random_extrusions;
#[path = "snapping_a_cones_apex_onto_the_origin.rs"]
pub mod snapping_a_cones_apex_onto_the_origin;
#[path = "the_point_across_a_circle_cut_at_a_pole_is_in_its_box.rs"]
pub mod the_point_across_a_circle_cut_at_a_pole_is_in_its_box;
#[path = "the_turned_pair_whose_union_lost_a_face_one_way_round.rs"]
pub mod the_turned_pair_whose_union_lost_a_face_one_way_round;
#[path = "the_volume_sweep_meshes_fine_enough_to_not_cross_itself.rs"]
pub mod the_volume_sweep_meshes_fine_enough_to_not_cross_itself;
#[path = "the_way_into_an_ellipse_arc_in_a_fuse.rs"]
pub mod the_way_into_an_ellipse_arc_in_a_fuse;
#[path = "three_cut_lines_through_one_corner_of_a_skew_pair.rs"]
pub mod three_cut_lines_through_one_corner_of_a_skew_pair;
#[path = "three_flat_seeds_that_fuse_one_way_round.rs"]
pub mod three_flat_seeds_that_fuse_one_way_round;
#[path = "turning_a_fuse.rs"]
pub mod turning_a_fuse;
#[path = "two_arcs_where_two_curved_surfaces_cross.rs"]
pub mod two_arcs_where_two_curved_surfaces_cross;
#[path = "two_bars_on_one_pipe_and_a_box_united_in_one_call.rs"]
pub mod two_bars_on_one_pipe_and_a_box_united_in_one_call;
#[path = "two_boxes_one_with_a_corner_moved_by_1e-300.rs"]
pub mod two_boxes_one_with_a_corner_moved_by_1e_300;
#[path = "two_cubes_corner_to_corner_turned_a_little.rs"]
pub mod two_cubes_corner_to_corner_turned_a_little;
#[path = "two_loops_named_apart_across_a_far_cube.rs"]
pub mod two_loops_named_apart_across_a_far_cube;
#[path = "two_loops_side_by_side_on_a_torus.rs"]
pub mod two_loops_side_by_side_on_a_torus;
#[path = "two_loops_that_run_the_same_way_round_a_tube.rs"]
pub mod two_loops_that_run_the_same_way_round_a_tube;
#[path = "two_meets_in_one_box_on_a_closed_crossing.rs"]
pub mod two_meets_in_one_box_on_a_closed_crossing;
#[path = "two_nodes_merged_on_a_closed_crossing_under_a_tolerant_policy.rs"]
pub mod two_nodes_merged_on_a_closed_crossing_under_a_tolerant_policy;
#[path = "two_skewed_combs_touching_along_one_line.rs"]
pub mod two_skewed_combs_touching_along_one_line;
#[path = "two_solids_in_the_same_place.rs"]
pub mod two_solids_in_the_same_place;
#[path = "two_spellings_of_one_pipe_mesh_each_face_on_its_own_side.rs"]
pub mod two_spellings_of_one_pipe_mesh_each_face_on_its_own_side;
#[path = "two_turned_cubes_a_rounding_apart.rs"]
pub mod two_turned_cubes_a_rounding_apart;
#[path = "validating_one_solid.rs"]
pub mod validating_one_solid;
#[path = "walking_the_model.rs"]
pub mod walking_the_model;
#[path = "what_a_cone_tip_costs_a_bound.rs"]
pub mod what_a_cone_tip_costs_a_bound;
#[path = "what_a_fuse_refusal_says_about_its_answer.rs"]
pub mod what_a_fuse_refusal_says_about_its_answer;
#[path = "what_a_leaning_cut_costs_the_floor_under_a_mesh.rs"]
pub mod what_a_leaning_cut_costs_the_floor_under_a_mesh;
#[path = "what_a_refused_edge_says.rs"]
pub mod what_a_refused_edge_says;
#[path = "what_a_tolerant_policy_costs_a_mesh.rs"]
pub mod what_a_tolerant_policy_costs_a_mesh;
#[path = "what_floor_a_bar_with_a_leaning_cap_is_given.rs"]
pub mod what_floor_a_bar_with_a_leaning_cap_is_given;
#[path = "what_is_closed_says_about_a_run_it_cannot_read.rs"]
pub mod what_is_closed_says_about_a_run_it_cannot_read;
#[path = "what_order_the_mesher_hands_its_triangles_back_in.rs"]
pub mod what_order_the_mesher_hands_its_triangles_back_in;
#[path = "what_the_fuse_does_to_a_seam_it_never_touched.rs"]
pub mod what_the_fuse_does_to_a_seam_it_never_touched;
#[path = "what_the_mesher_does_with_an_empty_walk_on_a_closed_edge.rs"]
pub mod what_the_mesher_does_with_an_empty_walk_on_a_closed_edge;
#[path = "what_the_solid_builder_refuses.rs"]
pub mod what_the_solid_builder_refuses;
#[path = "what_the_whole_sweep_says_about_a_two_step_wire.rs"]
pub mod what_the_whole_sweep_says_about_a_two_step_wire;
#[path = "when_it_cannot_tell.rs"]
pub mod when_it_cannot_tell;
#[path = "where_a_ball_and_a_ring_are_measured_from.rs"]
pub mod where_a_ball_and_a_ring_are_measured_from;
#[path = "where_a_lid_a_few_ulps_off_stops_being_one_plane.rs"]
pub mod where_a_lid_a_few_ulps_off_stops_being_one_plane;
#[path = "where_the_certified_dynamic_range_stops.rs"]
pub mod where_the_certified_dynamic_range_stops;
#[path = "whether_a_fine_face_is_handed_a_coarse_neighbours_edge.rs"]
pub mod whether_a_fine_face_is_handed_a_coarse_neighbours_edge;
#[path = "which_arc_of_a_cones_rim_a_side_is.rs"]
pub mod which_arc_of_a_cones_rim_a_side_is;
#[path = "which_side_a_reversed_wall_keeps.rs"]
pub mod which_side_a_reversed_wall_keeps;
#[path = "writing_and_reading.rs"]
pub mod writing_and_reading;

/// With `autotests` off, cargo builds a file in `tests/` only where this root
/// or a `[[test]]` names it, and says nothing about a file nothing names: its
/// tests just never run. A file named twice runs its tests twice.
#[test]
fn every_file_in_tests_is_built_into_exactly_one_test_binary() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    assert!(
        manifest
            .lines()
            .any(|line| line.trim() == "autotests = false"),
        "Cargo.toml no longer says `autotests = false`, so cargo builds every file \
         declared here a second time, as a binary of its own"
    );
    let mut times_built: BTreeMap<String, usize> = test_files_on_disk(&crate_dir.join("tests"))
        .into_iter()
        .map(|file| (file, 0))
        .collect();
    let built = test_targets_in(&manifest, crate_dir)
        .into_iter()
        .chain(files_declared_in(include_str!("merged.rs")));
    let mut not_in_tests = Vec::new();
    for file in built {
        match times_built.get_mut(&file) {
            Some(times) => *times += 1,
            None => not_in_tests.push(file),
        }
    }
    let never: Vec<&String> = times_built
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(f, _)| f)
        .collect();
    let twice: Vec<&String> = times_built
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|(f, _)| f)
        .collect();
    let declarations: String = never
        .iter()
        .map(|file| declaration_of(file.strip_prefix("tests/").unwrap()))
        .collect();
    assert!(
        never.is_empty() && twice.is_empty() && not_in_tests.is_empty(),
        "built by no binary, so its tests never run: {never:?}\n\
         declare each in tests/merged.rs, in its place by file name, as\n{declarations}\
         built more than once: {twice:?}\n\
         built but not a file directly under tests/: {not_in_tests:?}"
    );
}

/// A branch that adds a file puts its two lines between two that stay, so two
/// branches adding different files change different lines and merge without a
/// conflict, unless their files sort next to each other. Lines added at the
/// end would make every two such branches meet. `docs/decisions.md` §373.7.
#[test]
fn every_file_is_declared_as_a_path_and_a_pub_mod_in_order_of_its_name() {
    let lines: Vec<&str> = include_str!("merged.rs").lines().collect();
    let mut previous: Option<&str> = None;
    for (at, line) in lines.iter().enumerate() {
        let Some(file) = line
            .strip_prefix("#[path = \"")
            .and_then(|rest| rest.strip_suffix("\"]"))
        else {
            continue;
        };
        let written = format!("{line}\n{}\n", lines.get(at + 1).unwrap_or(&""));
        assert_eq!(
            written,
            declaration_of(file),
            "line {} of tests/merged.rs is not followed by its `pub mod`",
            at + 1
        );
        if let Some(previous) = previous {
            assert!(
                previous < file,
                "{file} is declared after {previous}, on line {}; each file goes in its \
                 place by name",
                at + 1
            );
        }
        previous = Some(file);
    }
}

/// The files cargo would find in `tests/` with `autotests` on: each `*.rs`
/// directly under it, and each `main.rs` one directory down. A helper such as
/// `tests/cones/mod.rs` is neither.
fn test_files_on_disk(tests_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(tests_dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        if path.is_dir() {
            if path.join("main.rs").is_file() {
                files.push(format!("tests/{name}/main.rs"));
            }
        } else if name.ends_with(".rs") {
            files.push(format!("tests/{name}"));
        }
    }
    files
}

/// The file each `[[test]]` in the manifest builds: its `path`, or, as cargo
/// infers it, `tests/<name>.rs` or else `tests/<name>/main.rs`.
fn test_targets_in(manifest: &str, crate_dir: &Path) -> Vec<String> {
    let mut targets: Vec<(Option<String>, Option<String>)> = Vec::new();
    let mut in_a_test = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_a_test = line == "[[test]]";
            if in_a_test {
                targets.push((None, None));
            }
        } else if in_a_test {
            let target = targets.last_mut().unwrap();
            if let Some(name) = quoted_value_of("name", line) {
                target.0 = Some(name);
            } else if let Some(path) = quoted_value_of("path", line) {
                target.1 = Some(path);
            }
        }
    }
    targets
        .into_iter()
        .map(|(name, path)| match (name, path) {
            (_, Some(path)) => path,
            (Some(name), None) if crate_dir.join(format!("tests/{name}.rs")).is_file() => {
                format!("tests/{name}.rs")
            }
            (Some(name), None) => format!("tests/{name}/main.rs"),
            (None, None) => panic!("a [[test]] in Cargo.toml has neither a name nor a path"),
        })
        .collect()
}

/// `value` from a manifest line `key = "value"`.
fn quoted_value_of(key: &str, line: &str) -> Option<String> {
    let rest = line
        .strip_prefix(key)?
        .trim_start()
        .strip_prefix('=')?
        .trim();
    Some(rest.strip_prefix('"')?.strip_suffix('"')?.to_owned())
}

/// The file each `#[path = "..."]` in this root names, as a path from the crate.
fn files_declared_in(root: &str) -> Vec<String> {
    root.lines()
        .filter_map(|line| line.trim().strip_prefix("#[path = \""))
        .map(|rest| format!("tests/{}", rest.strip_suffix("\"]").unwrap()))
        .collect()
}

/// The two lines that declare `file`, a path from `tests/`. A `-` in the name,
/// as in `..._1e-300.rs`, cannot be in a module's name, so it is a `_` there.
fn declaration_of(file: &str) -> String {
    let stem = file
        .strip_suffix("/main.rs")
        .or_else(|| file.strip_suffix(".rs"))
        .unwrap();
    let module = stem.replace('-', "_");
    ["#[path = \"", file, "\"]\npub mod ", &module, ";\n"].concat()
}
