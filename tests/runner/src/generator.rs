use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::test_cases::*;
use crate::themes::*;

pub struct Generator;

impl Generator {
    pub fn generate_all(base_dir: &Path) -> Result<()> {
        Self::create_case(
            base_dir,
            "1_page_letter",
            one_page_letter(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "financial_report",
            financial_report(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "infinite_canvas",
            infinite_canvas(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "typography_stress",
            typography_stress(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "table_100_rows",
            table_100_rows(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "nested_containers_depth_50",
            nested_containers(50),
            default_theme(),
        )?;

        // Additional fixtures (edge cases + coverage expansion)
        Self::create_case(
            base_dir,
            "unicode_stress",
            unicode_stress(),
            default_theme(),
        )?;
        Self::create_case(base_dir, "rtl_ltr_mix", rtl_ltr_mix(), default_theme())?;
        fs::write(base_dir.join("rtl_ltr_mix/font.txt"), "DejaVuSans.ttf\n")?;
        Self::create_case(
            base_dir,
            "wrapping_stress_long_token",
            wrapping_stress_long_token(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "grid_stack_nesting",
            grid_stack_nesting(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "modifier_overlap_matrix",
            modifier_overlap_matrix(),
            modifier_overlap_matrix_theme(),
        )?;
        Self::create_case(
            base_dir,
            "role_variant_primitives",
            role_variant_primitives(),
            role_variant_primitives_theme(),
        )?;
        Self::create_case(
            base_dir,
            "card_variants",
            card_variants(),
            card_variants_theme(),
        )?;
        Self::create_case(
            base_dir,
            "elevation_shadows",
            elevation_test(),
            card_variants_theme(),
        )?;
        Self::create_case(
            base_dir,
            "canvas_background_gradient",
            canvas_background_gradient(),
            canvas_background_gradient_theme(),
        )?;
        Self::create_case(
            base_dir,
            "overlay_glass_over_image",
            overlay_glass_over_image(),
            role_variant_primitives_theme(),
        )?;

        // Pagination edge cases (deterministic via explicit leaf sizes)
        Self::create_case(
            base_dir,
            "pagination_exact_fit_images",
            pagination_exact_fit_images(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "pagination_just_over_images",
            pagination_just_over_images(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "running_footer_page_numbers",
            running_footer_page_numbers(),
            running_footer_theme(),
        )?;
        Self::create_case_expected_error(
            base_dir,
            "pagination_oversize_first_item",
            pagination_oversize_first_item(),
            default_theme(),
            "UNSPLITTABLE_OVERFLOW",
        )?;

        // Layout coverage expansion
        Self::create_case(
            base_dir,
            "stack_direction_gap_matrix",
            stack_direction_gap_matrix(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "grid_track_matrix_basic",
            grid_track_matrix_basic(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "grid_fixed_overflow",
            grid_fixed_overflow(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "grid_nested_fr_rows",
            grid_nested_fr_rows(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "grid_partially_filled",
            grid_partially_filled(),
            default_theme(),
        )?;

        // Text wrapping edge cases
        Self::create_case(
            base_dir,
            "text_wrap_width_matrix",
            text_wrap_width_matrix(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "text_whitespace_newlines_matrix",
            text_whitespace_newlines_matrix(),
            default_theme(),
        )?;
        Self::create_case_expected_error(
            base_dir,
            "text_long_token_multiple_widths",
            text_long_token_multiple_widths(),
            default_theme(),
            "UNSPLITTABLE_OVERFLOW",
        )?;

        // Semantic list fixtures (flat list-item nodes with engine-derived markers)
        Self::create_case(
            base_dir,
            "semantic_lists_bullets_basic",
            semantic_lists_bullets_basic(),
            semantic_lists_theme(),
        )?;
        Self::create_case(
            base_dir,
            "semantic_lists_numbered_nested",
            semantic_lists_numbered_nested(),
            semantic_lists_theme(),
        )?;
        Self::create_case(
            base_dir,
            "semantic_lists_pagination_continues_numbers",
            semantic_lists_pagination_continues_numbers(),
            semantic_lists_theme(),
        )?;
        Self::create_case(
            base_dir,
            "semantic_lists_digit_boundary_no_reflow",
            semantic_lists_digit_boundary_no_reflow(),
            semantic_lists_theme(),
        )?;

        // Semantic code block fixtures (preformatted, whitespace-preserving, newline-only layout)
        Self::create_case(
            base_dir,
            "semantic_code_blocks_basic_verbatim",
            semantic_code_blocks_basic_verbatim(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "semantic_code_blocks_lines_vs_string_equivalence",
            semantic_code_blocks_lines_vs_string_equivalence(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "semantic_code_blocks_no_soft_wrap_long_line",
            semantic_code_blocks_no_soft_wrap_long_line(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "semantic_code_blocks_syntax_highlight_ranges",
            semantic_code_blocks_syntax_highlight_ranges(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "semantic_code_blocks_variant_dark_mode",
            semantic_code_blocks_variant_dark_mode(),
            default_theme(),
        )?;

        Self::create_case(
            base_dir,
            "math_display_frac",
            math_display_frac(),
            math_theme(),
        )?;
        fs::write(
            base_dir.join("math_display_frac/font.txt"),
            "NotoSansMath-Regular.ttf\n",
        )?;
        Self::create_case(
            base_dir,
            "math_display_sum",
            math_display_sum(),
            math_theme(),
        )?;
        fs::write(
            base_dir.join("math_display_sum/font.txt"),
            "NotoSansMath-Regular.ttf\n",
        )?;
        Self::create_case(
            base_dir,
            "math_display_left_frac",
            math_display_left_frac(),
            math_theme(),
        )?;
        fs::write(
            base_dir.join("math_display_left_frac/font.txt"),
            "NotoSansMath-Regular.ttf\n",
        )?;
        Self::create_case(
            base_dir,
            "math_display_pmatrix",
            math_display_pmatrix(),
            math_theme(),
        )?;
        fs::write(
            base_dir.join("math_display_pmatrix/font.txt"),
            "NotoSansMath-Regular.ttf\n",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "math_unknown_command",
            math_unknown_command(),
            math_theme(),
            "MATH_UNSUPPORTED",
        )?;
        fs::write(
            base_dir.join("math_unknown_command/font.txt"),
            "NotoSansMath-Regular.ttf\n",
        )?;

        // Modifier/style resolution that affects geometry (via theme-provided font_size patches)
        let modifier_manifest = modifier_precedence_matrix();
        Self::create_case(
            base_dir,
            "modifier_precedence_default",
            modifier_manifest.clone(),
            modifier_fontsize_theme_default_precedence(),
        )?;
        Self::create_case(
            base_dir,
            "modifier_precedence_custom",
            modifier_manifest,
            modifier_fontsize_theme_custom_precedence(),
        )?;
        Self::create_case(
            base_dir,
            "modifier_role_override",
            modifier_role_override(),
            modifier_role_override_theme(),
        )?;
        Self::create_case(
            base_dir,
            "modifier_adjacent_segmentation",
            modifier_adjacent_segmentation(),
            default_theme(),
        )?;

        // Leaf-node coverage (Image + TableReference)
        Self::create_case(
            base_dir,
            "leaf_image_table_reference",
            leaf_image_table_reference(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "leaf_image_scaling_by_width_paged",
            image_scaling_by_width_paged(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "leaf_image_scaling_by_height_bounded_container",
            image_scaling_by_height_bounded_container(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "leaf_table_reference_clamping_bounded_height",
            table_reference_clamping_bounded_height(),
            default_theme(),
        )?;

        // PageConfig edge cases
        Self::create_case(
            base_dir,
            "page_zero_margins",
            page_zero_margins(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "page_zero_content_width",
            page_zero_content_width(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "page_near_zero_margins",
            page_near_zero_margins(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "page_near_zero_content_width",
            page_near_zero_content_width(),
            default_theme(),
        )?;
        Self::create_case_expected_error(
            base_dir,
            "page_near_zero_content_height",
            page_near_zero_content_height(),
            default_theme(),
            "UNSPLITTABLE_OVERFLOW",
        )?;

        // Additional deterministic geometry-focused coverage
        Self::create_case(
            base_dir,
            "grid_fr_remainder_distribution",
            grid_fr_remainder_distribution(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "grid_cell_constraints_scale_image",
            grid_cell_constraints_scale_image(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "table_reference_constrained_in_cell",
            table_reference_constrained_in_cell(),
            default_theme(),
        )?;
        Self::create_case(
            base_dir,
            "text_line_height_modifier",
            text_line_height_modifier(),
            line_height_modifier_theme(),
        )?;

        // Expected-failure fixtures (edge cases that must error deterministically)
        Self::create_case_expected_error(
            base_dir,
            "error_modifier_limit_exceeded",
            error_modifier_limit_exceeded(),
            default_theme(),
            "modifier limit exceeded",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_modifier_range_invalid",
            error_modifier_range_invalid(),
            default_theme(),
            "invalid modifier range",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_modifier_range_not_on_char_boundary",
            error_modifier_range_not_on_char_boundary(),
            default_theme(),
            "not on UTF-8 char boundary",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_grid_too_many_children",
            error_grid_too_many_children(),
            default_theme(),
            "grid layout has more children",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_image_non_positive_size",
            error_image_non_positive_size(),
            default_theme(),
            "must have positive size",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_table_reference_non_positive_size",
            error_table_reference_non_positive_size(),
            default_theme(),
            "must have positive size",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_grid_fr_rows_unbounded",
            error_grid_fr_rows_unbounded(),
            default_theme(),
            "Cannot resolve fr tracks with infinite available size",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_page_config_negative_margin",
            error_page_config_negative_margin(),
            default_theme(),
            "Invalid PageConfig.margin",
        )?;
        Self::create_case_expected_error(
            base_dir,
            "error_page_config_negative_content_height",
            error_page_config_negative_content_height(),
            default_theme(),
            "top+bottom exceed height",
        )?;

        // Stress / depth
        Self::create_case(
            base_dir,
            "nested_containers_depth_200",
            nested_containers(200),
            default_theme(),
        )?;

        // Stress: Large page count for pagination determinism checks
        Self::create_case(
            base_dir,
            "pagination_large_page_count",
            pagination_large_page_count(),
            default_theme(),
        )?;

        Ok(())
    }

    fn create_case(
        base_dir: &Path,
        name: &str,
        manifest: k2f_core::Manifest,
        theme_json: String,
    ) -> Result<()> {
        let case_dir = base_dir.join(name);
        fs::create_dir_all(&case_dir)?;

        // Ensure stale expected-error marker isn't kept when converting a case back to a normal one.
        let expected_error_path = case_dir.join("expected_error.txt");
        if expected_error_path.exists() {
            let _ = fs::remove_file(&expected_error_path);
        }

        let content_path = case_dir.join("content.json");
        let json = serde_json::to_string_pretty(&manifest)?;
        fs::write(content_path, json)?;

        // Write theme.json (always overwrite so generator is a true source-of-truth)
        let theme_path = case_dir.join("theme.json");
        fs::write(theme_path, theme_json)?;

        println!("Generated case: {}", name);
        Ok(())
    }

    fn create_case_expected_error(
        base_dir: &Path,
        name: &str,
        manifest: k2f_core::Manifest,
        theme_json: String,
        expected_error_substring: &str,
    ) -> Result<()> {
        let case_dir = base_dir.join(name);
        fs::create_dir_all(&case_dir)?;

        let content_path = case_dir.join("content.json");
        let json = serde_json::to_string_pretty(&manifest)?;
        fs::write(content_path, json)?;

        let theme_path = case_dir.join("theme.json");
        fs::write(theme_path, theme_json)?;

        let expected_error_path = case_dir.join("expected_error.txt");
        fs::write(expected_error_path, expected_error_substring)?;

        println!("Generated case (expected error): {}", name);
        Ok(())
    }
}
