//! Most of rcad-geom's integration tests, built as one test binary.
//!
//! Every file directly under `tests/` used to be a binary of its own, and each
//! one linked rcad-geom again. Every worktree writes all of its test binaries,
//! since kache restores each one as a private copy, and those bytes are what is
//! wearing out the SSD (`docs/decisions.md` §373). Declared here as modules,
//! these files link rcad-geom once.
//!
//! Each file is declared with `#[path]` and stays where it is. A file reached
//! through `#[path]` looks for its own `mod support;` at `tests/support/mod.rs`,
//! just as it did as a crate root, so every line the record cites is still on
//! that line. `pub mod` keeps a file's `pub` items as live as they were when the
//! file was a crate root.
//!
//! A file the justfile or the record names by binary stays a binary of its own,
//! with a `[[test]]` in `Cargo.toml`, so that command still works.
//!
//! `autotests` is off, so a new file in `tests/` is built by nothing until it is
//! declared here, and `every_file_in_tests_is_built_into_exactly_one_test_binary`
//! fails until then, printing the two lines to add. They go in the file's place
//! by name, and the lane that adds the file adds them (`docs/decisions.md`
//! §373.7).

// A helper such as `tests/support/mod.rs` is a module of every file that
// declares it, so this binary holds one copy of it per file, as the separate
// binaries did.
#![allow(clippy::duplicate_mod)]

use std::path::Path;

mod one_binary;

#[path = "a_ball_a_rounding_off_a_pipes_axis.rs"]
pub mod a_ball_a_rounding_off_a_pipes_axis;
#[path = "a_bar_sliced_a_hair_inside_its_rim.rs"]
pub mod a_bar_sliced_a_hair_inside_its_rim;
#[path = "a_circle_built_from_dyadic_weights_is_exactly_a_circle.rs"]
pub mod a_circle_built_from_dyadic_weights_is_exactly_a_circle;
#[path = "a_circle_walked_between_two_spellings_of_one_cone.rs"]
pub mod a_circle_walked_between_two_spellings_of_one_cone;
#[path = "a_closed_run_whose_ends_differ_is_refused.rs"]
pub mod a_closed_run_whose_ends_differ_is_refused;
#[path = "a_coarse_walk_stays_on_the_crossing_instead_of_jumping_across_it.rs"]
pub mod a_coarse_walk_stays_on_the_crossing_instead_of_jumping_across_it;
#[path = "a_cone_crossed_by_a_cone_or_ball_on_its_axis.rs"]
pub mod a_cone_crossed_by_a_cone_or_ball_on_its_axis;
#[path = "a_corner_read_out_of_a_model_and_moved_is_a_box_around_where_it_lands.rs"]
pub mod a_corner_read_out_of_a_model_and_moved_is_a_box_around_where_it_lands;
#[path = "a_distance_is_the_same_shape_at_every_power_of_two.rs"]
pub mod a_distance_is_the_same_shape_at_every_power_of_two;
#[path = "a_far_corner_is_decided_where_squaring_it_overflows.rs"]
pub mod a_far_corner_is_decided_where_squaring_it_overflows;
#[path = "a_fuzz_draw_hands_its_provenance_back_through_the_door.rs"]
pub mod a_fuzz_draw_hands_its_provenance_back_through_the_door;
#[path = "a_hyperbola_arc_picks_its_branch_from_its_ends.rs"]
pub mod a_hyperbola_arc_picks_its_branch_from_its_ends;
#[path = "a_line_steeper_than_a_cones_rulings.rs"]
pub mod a_line_steeper_than_a_cones_rulings;
#[path = "a_normalize_with_no_rescale_rounds_wide_and_moves_with_the_length.rs"]
pub mod a_normalize_with_no_rescale_rounds_wide_and_moves_with_the_length;
#[path = "a_pair_of_parallel_pipes_answers_one_way_round_and_refuses_the_other.rs"]
pub mod a_pair_of_parallel_pipes_answers_one_way_round_and_refuses_the_other;
#[path = "a_pair_that_annihilates_with_no_second_route.rs"]
pub mod a_pair_that_annihilates_with_no_second_route;
#[path = "a_pipe_moved_by_a_speck_is_still_answered.rs"]
pub mod a_pipe_moved_by_a_speck_is_still_answered;
#[path = "a_pipe_too_thin_to_square_still_crosses_the_one_it_lies_on.rs"]
pub mod a_pipe_too_thin_to_square_still_crosses_the_one_it_lies_on;
#[path = "a_pipe_written_at_another_axis_scale_is_the_same_pipe.rs"]
pub mod a_pipe_written_at_another_axis_scale_is_the_same_pipe;
#[path = "a_point_exactly_on_a_circle_is_never_certified_off_it.rs"]
pub mod a_point_exactly_on_a_circle_is_never_certified_off_it;
#[path = "a_rotation_written_at_any_scale_is_the_same_rotation.rs"]
pub mod a_rotation_written_at_any_scale_is_the_same_rotation;
#[path = "a_round_trip_through_a_surfaces_own_two_coordinates.rs"]
pub mod a_round_trip_through_a_surfaces_own_two_coordinates;
#[path = "a_seam_plane_that_misses_the_ball_it_names.rs"]
pub mod a_seam_plane_that_misses_the_ball_it_names;
#[path = "a_turned_cone_cut_through_its_axis.rs"]
pub mod a_turned_cone_cut_through_its_axis;
#[path = "a_waved_through_walk_leaves_a_record.rs"]
pub mod a_waved_through_walk_leaves_a_record;
#[path = "an_arc_of_a_plane_cut_nearly_along_a_cones_ruling.rs"]
pub mod an_arc_of_a_plane_cut_nearly_along_a_cones_ruling;
#[path = "an_arc_of_a_plane_cut_nearly_along_a_pipes_axis.rs"]
pub mod an_arc_of_a_plane_cut_nearly_along_a_pipes_axis;
#[path = "an_enclosure_stays_within_a_few_ulps_of_the_point.rs"]
pub mod an_enclosure_stays_within_a_few_ulps_of_the_point;
#[path = "evaluating_a_spline_stays_inside_its_published_bound.rs"]
pub mod evaluating_a_spline_stays_inside_its_published_bound;
#[path = "every_bezier_segment_starts_where_the_last_one_ended.rs"]
pub mod every_bezier_segment_starts_where_the_last_one_ended;
#[path = "every_cone_or_torus_pair_the_fuzzer_draws_is_named_honestly.rs"]
pub mod every_cone_or_torus_pair_the_fuzzer_draws_is_named_honestly;
#[path = "every_generated_spline_is_one_the_kernel_accepts.rs"]
pub mod every_generated_spline_is_one_the_kernel_accepts;
#[path = "every_knot_vector_on_a_small_grid_is_judged_correctly.rs"]
pub mod every_knot_vector_on_a_small_grid_is_judged_correctly;
#[path = "every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs"]
pub mod every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly;
#[path = "every_point_along_a_crossing_is_on_both_surfaces.rs"]
pub mod every_point_along_a_crossing_is_on_both_surfaces;
#[path = "every_spelling_of_one_surface_names_the_same_crossing.rs"]
pub mod every_spelling_of_one_surface_names_the_same_crossing;
#[path = "how_a_plane_leans_against_a_pipe.rs"]
pub mod how_a_plane_leans_against_a_pipe;
#[path = "how_far_off_a_curved_surface_the_nearest_f64_can_sit.rs"]
pub mod how_far_off_a_curved_surface_the_nearest_f64_can_sit;
#[path = "how_far_the_marcher_strays_from_a_crossing_that_is_a_circle.rs"]
pub mod how_far_the_marcher_strays_from_a_crossing_that_is_a_circle;
#[path = "how_far_two_spellings_of_one_pipe_move_a_corner.rs"]
pub mod how_far_two_spellings_of_one_pipe_move_a_corner;
#[path = "how_finely_a_crossing_is_walked_is_about_its_size_not_its_place.rs"]
pub mod how_finely_a_crossing_is_walked_is_about_its_size_not_its_place;
#[path = "how_small_a_crossing_can_be_before_no_point_sits_on_it.rs"]
pub mod how_small_a_crossing_can_be_before_no_point_sits_on_it;
#[path = "inserting_a_knot_does_not_move_the_far_end.rs"]
pub mod inserting_a_knot_does_not_move_the_far_end;
#[path = "isometry_algebra.rs"]
pub mod isometry_algebra;
#[path = "one_answer_for_which_two_surfaces_a_run_is_between.rs"]
pub mod one_answer_for_which_two_surfaces_a_run_is_between;
#[path = "reading_a_distance_off_a_plane_stays_inside_its_bound.rs"]
pub mod reading_a_distance_off_a_plane_stays_inside_its_bound;
#[path = "scaling_a_spline_must_scale_its_answer_exactly.rs"]
pub mod scaling_a_spline_must_scale_its_answer_exactly;
#[path = "scaling_a_surface_must_not_change_its_answer.rs"]
pub mod scaling_a_surface_must_not_change_its_answer;
#[path = "serialization.rs"]
pub mod serialization;
#[path = "splitting_a_curve_leaves_both_halves_meeting_at_one_point.rs"]
pub mod splitting_a_curve_leaves_both_halves_meeting_at_one_point;
#[path = "the_angle_at_which_a_crossing_stops_having_a_direction.rs"]
pub mod the_angle_at_which_a_crossing_stops_having_a_direction;
#[path = "the_degeneracies_a_crossing_has_to_answer.rs"]
pub mod the_degeneracies_a_crossing_has_to_answer;
#[path = "the_dome_and_pipe_from_the_led_file_turn_by_a_real_amount.rs"]
pub mod the_dome_and_pipe_from_the_led_file_turn_by_a_real_amount;
#[path = "the_pair_at_1e16_that_answers_one_way_round.rs"]
pub mod the_pair_at_1e16_that_answers_one_way_round;
#[path = "the_same_pair_turned_names_the_same_kind_of_crossing.rs"]
pub mod the_same_pair_turned_names_the_same_kind_of_crossing;
#[path = "the_seven_shapes_a_plane_cuts_a_cone_in.rs"]
pub mod the_seven_shapes_a_plane_cuts_a_cone_in;
#[path = "the_spellings_a_round_surface_still_has.rs"]
pub mod the_spellings_a_round_surface_still_has;
#[path = "the_step_between_two_pipe_axes_taken_scaled.rs"]
pub mod the_step_between_two_pipe_axes_taken_scaled;
#[path = "the_step_between_two_pipe_axes_that_rounds_to_nothing.rs"]
pub mod the_step_between_two_pipe_axes_that_rounds_to_nothing;
#[path = "the_turns_of_a_cube_are_said_exactly.rs"]
pub mod the_turns_of_a_cube_are_said_exactly;
#[path = "the_two_arcs_of_a_cap_rim_cut_a_hair_apart.rs"]
pub mod the_two_arcs_of_a_cap_rim_cut_a_hair_apart;
#[path = "the_two_arcs_of_a_long_cone_ellipse_cut_a_hair_apart.rs"]
pub mod the_two_arcs_of_a_long_cone_ellipse_cut_a_hair_apart;
#[path = "the_two_sites_that_threw_an_exact_answer_away.rs"]
pub mod the_two_sites_that_threw_an_exact_answer_away;
#[path = "the_two_spellings_of_a_placed_cones_apex.rs"]
pub mod the_two_spellings_of_a_placed_cones_apex;
#[path = "three_more_places_a_crossings_two_pieces_cannot_be_told_apart.rs"]
pub mod three_more_places_a_crossings_two_pieces_cannot_be_told_apart;
#[path = "three_surfaces_name_the_pipe_the_caller_wrote.rs"]
pub mod three_surfaces_name_the_pipe_the_caller_wrote;
#[path = "two_loops_either_side_of_a_pipe.rs"]
pub mod two_loops_either_side_of_a_pipe;
#[path = "two_pipes_far_from_the_origin_name_their_crossing_the_same_both_ways_round.rs"]
pub mod two_pipes_far_from_the_origin_name_their_crossing_the_same_both_ways_round;
#[path = "two_pipes_resting_against_each_other_touch_at_one_point.rs"]
pub mod two_pipes_resting_against_each_other_touch_at_one_point;
#[path = "two_spheres_a_hair_apart_are_one_sphere_and_a_second_surface.rs"]
pub mod two_spheres_a_hair_apart_are_one_sphere_and_a_second_surface;
#[path = "walking_the_two_circles_a_cone_and_a_pipe_cut.rs"]
pub mod walking_the_two_circles_a_cone_and_a_pipe_cut;
#[path = "what_a_geometry_refusal_says_about_its_answer.rs"]
pub mod what_a_geometry_refusal_says_about_its_answer;
#[path = "what_makes_a_huge_pipe_against_a_thin_one_slow_to_walk.rs"]
pub mod what_makes_a_huge_pipe_against_a_thin_one_slow_to_walk;
#[path = "what_moving_a_refusals_surfaces_costs.rs"]
pub mod what_moving_a_refusals_surfaces_costs;
#[path = "what_the_dynamic_range_check_is_a_margin_against.rs"]
pub mod what_the_dynamic_range_check_is_a_margin_against;
#[path = "what_twice_the_room_buys_a_walk_at_its_step_cap.rs"]
pub mod what_twice_the_room_buys_a_walk_at_its_step_cap;
#[path = "where_a_closed_crossing_starts_is_a_place_or_a_refusal.rs"]
pub mod where_a_closed_crossing_starts_is_a_place_or_a_refusal;
#[path = "where_a_coaxial_cone_and_pipe_cross.rs"]
pub mod where_a_coaxial_cone_and_pipe_cross;
#[path = "where_a_line_meets_a_torus.rs"]
pub mod where_a_line_meets_a_torus;
#[path = "where_a_plane_and_two_round_surfaces_meet.rs"]
pub mod where_a_plane_and_two_round_surfaces_meet;
#[path = "where_a_surfaces_own_two_coordinates_land.rs"]
pub mod where_a_surfaces_own_two_coordinates_land;
#[path = "where_an_ellipse_a_plane_cuts_on_a_cone_is_centred.rs"]
pub mod where_an_ellipse_a_plane_cuts_on_a_cone_is_centred;
#[path = "where_two_leaning_pipes_far_out_cross.rs"]
pub mod where_two_leaning_pipes_far_out_cross;
#[path = "where_two_side_by_side_pipes_stop_being_told_apart.rs"]
pub mod where_two_side_by_side_pipes_stop_being_told_apart;
#[path = "whether_a_tolerant_policy_reaches_the_half_spaces_on_a_run.rs"]
pub mod whether_a_tolerant_policy_reaches_the_half_spaces_on_a_run;
#[path = "why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes.rs"]
pub mod why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes;

#[test]
fn every_file_in_tests_is_built_into_exactly_one_test_binary() {
    one_binary::every_file_in_tests_is_built_into_exactly_one_test_binary(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        include_str!("merged.rs"),
    );
}

#[test]
fn every_file_is_declared_as_a_path_and_a_pub_mod_in_order_of_its_name() {
    one_binary::every_file_is_declared_as_a_path_and_a_pub_mod_in_order_of_its_name(include_str!(
        "merged.rs"
    ));
}
