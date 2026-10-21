//! Most of rcad-step's integration tests, built as one test binary.
//!
//! Every file directly under `tests/` used to be a binary of its own, and each
//! one linked the reader, the writer and the kernel under them again. Every
//! worktree writes all of its test binaries, since kache restores each one as a
//! private copy, and those bytes are what is wearing out the SSD
//! (`docs/decisions.md` §373). Declared here as modules, these files link them
//! once.
//!
//! Each file is declared with `#[path]` and stays where it is. A file reached
//! through `#[path]` looks for its own `mod written_by_hand;` at
//! `tests/written_by_hand/mod.rs`, just as it did as a crate root, so every line
//! the record cites is still on that line. `pub mod` keeps a file's `pub` items
//! as live as they were when the file was a crate root.
//!
//! A file the justfile or the record names by binary stays a binary of its own,
//! with a `[[test]]` in `Cargo.toml`, so that command still works.
//!
//! `autotests` is off, so a new file in `tests/` is built by nothing until it is
//! declared here, and `every_file_in_tests_is_built_into_exactly_one_test_binary`
//! fails until then, printing the two lines to add. They go in the file's place
//! by name, and the lane that adds the file adds them (`docs/decisions.md`
//! §373.7).

// A helper such as `tests/written_by_hand/mod.rs` is a module of every file that
// declares it, so this binary holds one copy of it per file, as the separate
// binaries did.
#![allow(clippy::duplicate_mod)]

use std::path::Path;

#[path = "../../rcad-topo/tests/cones/mod.rs"]
mod cones;
#[path = "../../rcad-geom/tests/one_binary/mod.rs"]
mod one_binary;

#[path = "a_ball_resting_in_a_cone_rounds_its_tip.rs"]
pub mod a_ball_resting_in_a_cone_rounds_its_tip;
#[path = "a_ball_rim_cornered_where_it_runs_along_a_meridian.rs"]
pub mod a_ball_rim_cornered_where_it_runs_along_a_meridian;
#[path = "a_ball_turned_about_z_is_written.rs"]
pub mod a_ball_turned_about_z_is_written;
#[path = "a_bar_capped_by_a_dome_of_its_own_radius_imports.rs"]
pub mod a_bar_capped_by_a_dome_of_its_own_radius_imports;
#[path = "a_circle_a_file_draws_round_in_one_edge.rs"]
pub mod a_circle_a_file_draws_round_in_one_edge;
#[path = "a_circle_between_two_flat_faces_that_one_does_not_hold.rs"]
pub mod a_circle_between_two_flat_faces_that_one_does_not_hold;
#[path = "a_circle_imprinted_on_a_flat_face_keeps_its_bend.rs"]
pub mod a_circle_imprinted_on_a_flat_face_keeps_its_bend;
#[path = "a_circle_leaning_off_a_torus_counted_once.rs"]
pub mod a_circle_leaning_off_a_torus_counted_once;
#[path = "a_circle_the_file_puts_off_its_pipe_past_the_allowance.rs"]
pub mod a_circle_the_file_puts_off_its_pipe_past_the_allowance;
#[path = "a_circle_the_file_puts_on_a_pipe_is_off_it_in_exact_arithmetic.rs"]
pub mod a_circle_the_file_puts_on_a_pipe_is_off_it_in_exact_arithmetic;
#[path = "a_closed_circle_between_one_cone_written_twice.rs"]
pub mod a_closed_circle_between_one_cone_written_twice;
#[path = "a_closed_circle_between_one_curved_surface_written_twice.rs"]
pub mod a_closed_circle_between_one_curved_surface_written_twice;
#[path = "a_closed_circle_between_one_torus_written_twice.rs"]
pub mod a_closed_circle_between_one_torus_written_twice;
#[path = "a_closed_ellipse_between_one_pipe_written_twice.rs"]
pub mod a_closed_ellipse_between_one_pipe_written_twice;
#[path = "a_curved_face_the_reader_finds_off_its_surface.rs"]
pub mod a_curved_face_the_reader_finds_off_its_surface;
#[path = "a_dome_on_a_flat_disc_imports.rs"]
pub mod a_dome_on_a_flat_disc_imports;
#[path = "a_file_bounded_by_two_half_circles.rs"]
pub mod a_file_bounded_by_two_half_circles;
#[path = "a_flared_bar_reads_whichever_way_its_pipe_points.rs"]
pub mod a_flared_bar_reads_whichever_way_its_pipe_points;
#[path = "a_flat_cap_is_cut_up_the_same_after_its_own_file.rs"]
pub mod a_flat_cap_is_cut_up_the_same_after_its_own_file;
#[path = "a_fuse_read_back_from_rounded_text.rs"]
pub mod a_fuse_read_back_from_rounded_text;
#[path = "a_mitre_ellipse_between_two_thin_pipes.rs"]
pub mod a_mitre_ellipse_between_two_thin_pipes;
#[path = "a_mitre_whose_centre_the_file_writes_off_its_pipe.rs"]
pub mod a_mitre_whose_centre_the_file_writes_off_its_pipe;
#[path = "a_quarter_bend_turned_about_x_measures_after_its_file.rs"]
pub mod a_quarter_bend_turned_about_x_measures_after_its_file;
#[path = "a_real_file_written_out_and_read_back_is_the_same_solid.rs"]
pub mod a_real_file_written_out_and_read_back_is_the_same_solid;
#[path = "a_rim_square_to_the_way_its_surface_winds.rs"]
pub mod a_rim_square_to_the_way_its_surface_winds;
#[path = "a_rim_through_both_poles_measures.rs"]
pub mod a_rim_through_both_poles_measures;
#[path = "a_run_between_two_curved_faces_is_named_alike_in_either_order.rs"]
pub mod a_run_between_two_curved_faces_is_named_alike_in_either_order;
#[path = "a_second_reader_reads_what_this_writer_wrote.rs"]
pub mod a_second_reader_reads_what_this_writer_wrote;
#[path = "a_solid_the_writer_harness_judges_alone.rs"]
pub mod a_solid_the_writer_harness_judges_alone;
#[path = "a_solid_written_by_hand_becomes_a_model.rs"]
pub mod a_solid_written_by_hand_becomes_a_model;
#[path = "a_solid_written_out_and_read_back_is_the_same_solid.rs"]
pub mod a_solid_written_out_and_read_back_is_the_same_solid;
#[path = "a_z2_led_corner_off_its_face.rs"]
pub mod a_z2_led_corner_off_its_face;
#[path = "an_led_dome_seam_is_cut_once.rs"]
pub mod an_led_dome_seam_is_cut_once;
#[path = "an_led_dome_wire_lies_flat.rs"]
pub mod an_led_dome_wire_lies_flat;
#[path = "every_file_in_the_corpus_reads.rs"]
pub mod every_file_in_the_corpus_reads;
#[path = "every_file_the_reader_fuzzer_kept_is_read_again.rs"]
pub mod every_file_the_reader_fuzzer_kept_is_read_again;
#[path = "every_model_the_writer_fuzzer_kept_is_written_again.rs"]
pub mod every_model_the_writer_fuzzer_kept_is_written_again;
#[path = "how_often_a_real_file_imports.rs"]
pub mod how_often_a_real_file_imports;
#[path = "instances_can_be_named_and_stored.rs"]
pub mod instances_can_be_named_and_stored;
#[path = "no_cone_face_arrives_out_of_the_corpus.rs"]
pub mod no_cone_face_arrives_out_of_the_corpus;
#[path = "one_wall_whose_normal_is_spelled_two_ways.rs"]
pub mod one_wall_whose_normal_is_spelled_two_ways;
#[path = "styling_is_never_among_the_shapes_walked_past.rs"]
pub mod styling_is_never_among_the_shapes_walked_past;
#[path = "the_reader_fuzz_dictionary_names_what_the_reader_reads.rs"]
pub mod the_reader_fuzz_dictionary_names_what_the_reader_reads;
#[path = "the_same_file_read_twice_by_two_readers.rs"]
pub mod the_same_file_read_twice_by_two_readers;
#[path = "the_same_part_written_at_another_scale.rs"]
pub mod the_same_part_written_at_another_scale;
#[path = "the_shapes_a_part_21_file_takes.rs"]
pub mod the_shapes_a_part_21_file_takes;
#[path = "two_turns_that_write_the_same_placement.rs"]
pub mod two_turns_that_write_the_same_placement;
#[path = "what_the_depth_probe_finds_behind_the_first_refusal.rs"]
pub mod what_the_depth_probe_finds_behind_the_first_refusal;
#[path = "what_the_mapping_refuses_and_by_what_name.rs"]
pub mod what_the_mapping_refuses_and_by_what_name;
#[path = "what_the_reader_refuses_and_by_what_name.rs"]
pub mod what_the_reader_refuses_and_by_what_name;
#[path = "whether_a_tolerant_policy_buys_anything_a_certified_one_does_not.rs"]
pub mod whether_a_tolerant_policy_buys_anything_a_certified_one_does_not;
#[path = "which_way_a_cap_circle_is_walked.rs"]
pub mod which_way_a_cap_circle_is_walked;
#[path = "which_way_a_torus_faces_normal_points.rs"]
pub mod which_way_a_torus_faces_normal_points;

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
