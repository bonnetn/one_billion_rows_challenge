#![feature(portable_simd)]
#![feature(mpmc_channel)]
#![warn(
    clippy::pedantic,
    clippy::cargo_common_metadata,
    clippy::negative_feature_names,
    clippy::redundant_feature_names,
    clippy::wildcard_dependencies,
    clippy::alloc_instead_of_core,
    clippy::allow_attributes_without_reason,
    clippy::as_conversions,
    clippy::as_pointer_underscore,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::cfg_not_test,
    clippy::clone_on_ref_ptr,
    clippy::cognitive_complexity,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::default_numeric_fallback,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::disallowed_script_idents,
    clippy::doc_include_without_cfg,
    clippy::doc_paragraphs_missing_punctuation,
    clippy::else_if_without_else,
    clippy::empty_drop,
    clippy::empty_enum_variants_with_brackets,
    clippy::empty_structs_with_brackets,
    clippy::error_impl_error,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::exit,
    clippy::field_scoped_visibility_modifiers,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::get_unwrap,
    clippy::infinite_loop,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::map_with_unused_argument_over_ranges,
    clippy::mem_forget,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::missing_inline_in_public_items,
    clippy::mixed_read_write_in_expression,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::needless_raw_strings,
    clippy::non_ascii_literal,
    clippy::non_zero_suggestions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::pathbuf_init_then_push,
    clippy::pattern_type_mismatch,
    clippy::pointer_format,
    clippy::precedence_bits,
    clippy::print_stderr,
    clippy::pub_without_shorthand,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_test_prefix,
    clippy::redundant_type_annotations,
    clippy::renamed_function_params,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::return_and_then,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::semicolon_outside_block,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_lit_chars_any,
    clippy::string_slice,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::unused_result_ok,
    clippy::unused_trait_names,
    clippy::unwrap_used,
    clippy::verbose_file_reads,
    clippy::wildcard_enum_match_arm
)]

use std::{path::Path, time::Instant};

use anyhow::Result;

mod challenge;
mod generate;
mod station_name;
mod stats;
mod value;

fn main() -> Result<()> {
    let samples_path = Path::new("measurements.txt");
    if std::env::args().nth(1).is_some() {
        generate::generate_samples(samples_path)?;
    } else {
        let start = Instant::now();
        challenge::run(samples_path)?;
        let end = Instant::now();
        println!("Time taken: {:?}", end.duration_since(start));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    const BASE_PATH: &str = "1brc/src/test/resources/samples";

    macro_rules! sample_tests {
        ($( $name:ident => $sample:expr ),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let base_path = std::path::Path::new(BASE_PATH);

                    let input_path = base_path
                        .join($sample)
                        .with_extension("txt");

                    let output_path = base_path
                        .join($sample)
                        .with_extension("out");

                    let expected = std::fs::read_to_string(&output_path).unwrap();

                    let got = challenge::run(&input_path).unwrap();
                    assert_eq!(
                        got,
                        expected,
                        "Invalid output: {}",
                        input_path.display()
                    );
                }
            )*
        };
    }

    sample_tests!(
        measurements_1 => "measurements-1",
        measurements_10 => "measurements-10",
        measurements_10000_unique_keys => "measurements-10000-unique-keys",
        measurements_2 => "measurements-2",
        measurements_20 => "measurements-20",
        measurements_3 => "measurements-3",
        measurements_boundaries => "measurements-boundaries",
        measurements_complex_utf8 => "measurements-complex-utf8",
        measurements_dot => "measurements-dot",
        measurements_rounding => "measurements-rounding",
        measurements_short => "measurements-short",
        measurements_shortest => "measurements-shortest",
    );

    #[test]
    fn test_measurements() {
        let expected = std::fs::read_to_string(Path::new("measurements.out")).unwrap();

        let samples_path = Path::new("measurements.txt");
        generate::generate_samples(samples_path).unwrap();
        let result = challenge::run(samples_path).unwrap();

        assert_eq!(result, expected);
    }
}
