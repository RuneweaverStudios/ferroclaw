//! Test module for the color palette.
//!
//! This module ensures that all colors and styles are properly defined
//! and can be imported and used without issues.

#[cfg(test)]
mod tests {
    use super::super::colors::*;

    #[test]
    fn test_all_colors_defined() {
        // Background colors
        let _bg_primary = BG_PRIMARY;
        let _bg_secondary = BG_SECONDARY;
        let _bg_tertiary = BG_TERTIARY;

        // Foreground colors
        let _fg_primary = FG_PRIMARY;
        let _fg_secondary = FG_SECONDARY;
        let _fg_tertiary = FG_TERTIARY;

        // Accent colors
        let _accent_primary = ACCENT_PRIMARY;
        let _accent_secondary = ACCENT_SECONDARY;
        let _accent_tertiary = ACCENT_TERTIARY;
        let _accent_amber = ACCENT_AMBER;
        let _accent_orange = ACCENT_ORANGE;
        let _accent_assistant = ACCENT_ASSISTANT;

        // Status colors
        let _status_error = STATUS_ERROR;
        let _status_warning = STATUS_WARNING;
        let _status_success = STATUS_SUCCESS;
        let _status_info = STATUS_INFO;

        // Code colors
        let _code_foreground = CODE_FOREGROUND;
        let _code_background = CODE_BACKGROUND;

        // Border colors
        let _border_primary = BORDER_PRIMARY;
        let _border_accent = BORDER_ACCENT;
    }

    #[test]
    fn test_all_style_functions() {
        // All style functions should be callable without panicking
        let _default = style_default();
        let _secondary = style_secondary();
        let _tertiary = style_tertiary();
        let _accent = style_accent();
        let _error = style_error();
        let _warning = style_warning();
        let _success = style_success();
        let _info = style_info();
        let _code = style_code();
        let _user_message = style_user_message();
        let _assistant_message = style_assistant_message();
        let _tool_call = style_tool_call();
        let _tool_result_success = style_tool_result_success();
        let _tool_result_error = style_tool_result_error();
        let _header = style_header();
        let _footer = style_footer();
        let _selected = style_selected();
        let _input = style_input();
        let _status_running = style_status_running();
        let _status_idle = style_status_idle();
    }

    #[test]
    fn test_get_color() {
        // Test getting colors by name
        assert!(get_color("bg_primary").is_some());
        assert!(get_color("bg-primary").is_some());
        assert!(get_color("accent_primary").is_some());
        assert!(get_color("error").is_some());
        assert!(get_color("invalid_color_name").is_none());
    }

    #[test]
    fn test_get_style() {
        // Test getting styles by semantic name
        assert!(get_style("default").is_some());
        assert!(get_style("accent").is_some());
        assert!(get_style("error").is_some());
        assert!(get_style("user_message").is_some());
        assert!(get_style("invalid_style_name").is_none());
    }

    #[test]
    fn test_legacy_compatibility() {
        // Ensure legacy color constants are defined
        let _teal = TEAL;
        let _amber = AMBER;
        let _emerald = EMERALD;
        let _cyan = CYAN;
        let _gray_300 = GRAY_300;
        let _gray_500 = GRAY_500;
        let _red_light = RED_LIGHT;
        let _hermes_orange = HERMES_ORANGE;
        let _hermes_assist = HERMES_ASSIST;
        let _tile_bg = TILE_BG;
        let _tile_border = TILE_BORDER;
        let _tile_header = TILE_HEADER;
        let _code_fg = CODE_FG;
    }
}
