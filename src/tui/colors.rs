//! Color palette module for TUI components.
//!
//! Provides a consistent color scheme across all TUI implementations to ensure
//! visual harmony and user experience. Colors are defined using RGB values for
//! precise control and can be easily modified to create different themes.

use ratatui::style::Color;

// ============================================================================
// PRIMARY PALETTE
// ============================================================================

/// Main background color - dark navy/slate for modern terminal look
pub const BG_PRIMARY: Color = Color::Rgb(19, 19, 26);

/// Secondary background color - slightly lighter for panel differentiation
pub const BG_SECONDARY: Color = Color::Rgb(22, 22, 31);

/// Tertiary background color - for headers, footers, and special panels
pub const BG_TERTIARY: Color = Color::Rgb(30, 30, 42);

/// Primary foreground color - light gray for main text
pub const FG_PRIMARY: Color = Color::Rgb(209, 213, 219);

/// Secondary foreground color - medium gray for less prominent text
pub const FG_SECONDARY: Color = Color::Rgb(107, 114, 128);

/// Tertiary foreground color - dim gray for hints and metadata
pub const FG_TERTIARY: Color = Color::Rgb(75, 85, 99);

// ============================================================================
// ACCENT COLORS
// ============================================================================

/// Primary accent color - teal for branding and important elements
pub const ACCENT_PRIMARY: Color = Color::Rgb(0, 212, 170);

/// Secondary accent color - cyan for highlights and links
pub const ACCENT_SECONDARY: Color = Color::Rgb(103, 232, 249);

/// Tertiary accent color - emerald for success states
pub const ACCENT_TERTIARY: Color = Color::Rgb(52, 211, 153);

/// Special accent color - amber for warnings and pending states
pub const ACCENT_AMBER: Color = Color::Rgb(251, 191, 36);

/// Brand accent color - orange for user messages and key interactions
pub const ACCENT_ORANGE: Color = Color::Rgb(255, 107, 53);

/// Assistant accent color - green for assistant messages
pub const ACCENT_ASSISTANT: Color = Color::Rgb(74, 222, 128);

// ============================================================================
// STATUS COLORS
// ============================================================================

/// Error color - soft red for errors and failures
pub const STATUS_ERROR: Color = Color::Rgb(252, 165, 165);

/// Warning color - soft amber for warnings
pub const STATUS_WARNING: Color = Color::Rgb(253, 186, 116);

/// Success color - soft green for success states
pub const STATUS_SUCCESS: Color = Color::Rgb(134, 239, 172);

/// Info color - soft blue for informational messages
pub const STATUS_INFO: Color = Color::Rgb(147, 197, 253);

/// Pending color - yellow for in-progress states
pub const STATUS_PENDING: Color = Color::Rgb(250, 204, 21);

/// In-progress color - amber for active operations
pub const STATUS_IN_PROGRESS: Color = Color::Rgb(251, 191, 36);

// ============================================================================
// CODE COLORS
// ============================================================================

/// Code foreground color - light cyan for code blocks
pub const CODE_FOREGROUND: Color = Color::Rgb(207, 250, 254);

/// Code background color - very dark for code blocks
pub const CODE_BACKGROUND: Color = Color::Rgb(15, 23, 42);

/// Code comment color - dim gray for comments
pub const CODE_COMMENT: Color = Color::Rgb(100, 116, 139);

/// Code string color - light green for string literals
pub const CODE_STRING: Color = Color::Rgb(134, 239, 172);

/// Code keyword color - light purple for keywords
pub const CODE_KEYWORD: Color = Color::Rgb(196, 181, 253);

/// Code function color - light blue for function names
pub const CODE_FUNCTION: Color = Color::Rgb(96, 165, 250);

/// Code number color - light orange for numeric literals
pub const CODE_NUMBER: Color = Color::Rgb(253, 186, 116);

// ============================================================================
// BORDER COLORS
// ============================================================================

/// Primary border color - matches secondary background for subtle borders
pub const BORDER_PRIMARY: Color = Color::Rgb(30, 30, 42);

/// Secondary border color - slightly darker for emphasis
pub const BORDER_SECONDARY: Color = Color::Rgb(51, 65, 85);

/// Accent border color - teal for focused elements
pub const BORDER_ACCENT: Color = Color::Rgb(0, 212, 170);

/// Error border color - red for error states
pub const BORDER_ERROR: Color = Color::Rgb(252, 165, 165);

// ============================================================================
// LEGACY COMPATIBILITY
// ============================================================================

/// Legacy: Teal (mapped to ACCENT_PRIMARY)
pub const TEAL: Color = ACCENT_PRIMARY;

/// Legacy: Amber (mapped to ACCENT_AMBER)
pub const AMBER: Color = ACCENT_AMBER;

/// Legacy: Emerald (mapped to ACCENT_TERTIARY)
pub const EMERALD: Color = ACCENT_TERTIARY;

/// Legacy: Cyan (mapped to ACCENT_SECONDARY)
pub const CYAN: Color = ACCENT_SECONDARY;

/// Legacy: Gray 300 (mapped to FG_PRIMARY)
pub const GRAY_300: Color = FG_PRIMARY;

/// Legacy: Gray 500 (mapped to FG_SECONDARY)
pub const GRAY_500: Color = FG_SECONDARY;

/// Legacy: Light red (mapped to STATUS_ERROR)
pub const RED_LIGHT: Color = STATUS_ERROR;

/// Legacy: Hermes orange (mapped to ACCENT_ORANGE)
pub const HERMES_ORANGE: Color = ACCENT_ORANGE;

/// Legacy: Hermes assistant green (mapped to ACCENT_ASSISTANT)
pub const HERMES_ASSIST: Color = ACCENT_ASSISTANT;

/// Legacy: Tile background (mapped to BG_PRIMARY)
pub const TILE_BG: Color = BG_PRIMARY;

/// Legacy: Tile border (mapped to BORDER_PRIMARY)
pub const TILE_BORDER: Color = BORDER_PRIMARY;

/// Legacy: Tile header (mapped to BG_SECONDARY)
pub const TILE_HEADER: Color = BG_SECONDARY;

/// Legacy: Code foreground (mapped to CODE_FOREGROUND)
pub const CODE_FG: Color = CODE_FOREGROUND;

// ============================================================================
// PRE-DEFINED STYLE COMBINATIONS
// ============================================================================

use ratatui::style::{Modifier, Style};

/// Style for default text
pub fn style_default() -> Style {
    Style::default().fg(FG_PRIMARY).bg(BG_PRIMARY)
}

/// Style for secondary/dim text
pub fn style_secondary() -> Style {
    Style::default().fg(FG_SECONDARY).bg(BG_PRIMARY)
}

/// Style for tertiary/hint text
pub fn style_tertiary() -> Style {
    Style::default()
        .fg(FG_TERTIARY)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::ITALIC)
}

/// Style for accent text
pub fn style_accent() -> Style {
    Style::default()
        .fg(ACCENT_PRIMARY)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for error text
pub fn style_error() -> Style {
    Style::default()
        .fg(STATUS_ERROR)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for warning text
pub fn style_warning() -> Style {
    Style::default().fg(STATUS_WARNING).bg(BG_PRIMARY)
}

/// Style for success text
pub fn style_success() -> Style {
    Style::default().fg(STATUS_SUCCESS).bg(BG_PRIMARY)
}

/// Style for info text
pub fn style_info() -> Style {
    Style::default()
        .fg(STATUS_INFO)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::ITALIC)
}

/// Style for code blocks
pub fn style_code() -> Style {
    Style::default().fg(CODE_FOREGROUND).bg(CODE_BACKGROUND)
}

/// Style for borders (primary)
pub fn style_border_primary() -> Style {
    Style::default().fg(BORDER_PRIMARY).bg(BG_PRIMARY)
}

/// Style for borders (accent)
pub fn style_border_accent() -> Style {
    Style::default().fg(BORDER_ACCENT).bg(BG_PRIMARY)
}

/// Style for user messages
pub fn style_user_message() -> Style {
    Style::default()
        .fg(ACCENT_ORANGE)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for assistant messages
pub fn style_assistant_message() -> Style {
    Style::default()
        .fg(ACCENT_ASSISTANT)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for tool calls
pub fn style_tool_call() -> Style {
    Style::default()
        .fg(ACCENT_AMBER)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for tool results (success)
pub fn style_tool_result_success() -> Style {
    Style::default().fg(ACCENT_TERTIARY).bg(BG_PRIMARY)
}

/// Style for tool results (error)
pub fn style_tool_result_error() -> Style {
    Style::default()
        .fg(STATUS_ERROR)
        .bg(BG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for headers
pub fn style_header() -> Style {
    Style::default()
        .fg(FG_PRIMARY)
        .bg(BG_SECONDARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for footers
pub fn style_footer() -> Style {
    Style::default().fg(FG_SECONDARY).bg(BG_SECONDARY)
}

/// Style for selected items
pub fn style_selected() -> Style {
    Style::default()
        .fg(FG_PRIMARY)
        .bg(BG_TERTIARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for input fields
pub fn style_input() -> Style {
    Style::default().fg(FG_PRIMARY).bg(BG_PRIMARY)
}

/// Style for status indicators (running)
pub fn style_status_running() -> Style {
    Style::default()
        .fg(ACCENT_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for status indicators (idle)
pub fn style_status_idle() -> Style {
    Style::default().fg(FG_SECONDARY)
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get a color by name (for dynamic color selection)
pub fn get_color(name: &str) -> Option<Color> {
    match name.to_lowercase().as_str() {
        "bg_primary" | "bg-primary" => Some(BG_PRIMARY),
        "bg_secondary" | "bg-secondary" => Some(BG_SECONDARY),
        "bg_tertiary" | "bg-tertiary" => Some(BG_TERTIARY),
        "fg_primary" | "fg-primary" => Some(FG_PRIMARY),
        "fg_secondary" | "fg-secondary" => Some(FG_SECONDARY),
        "fg_tertiary" | "fg-tertiary" => Some(FG_TERTIARY),
        "accent_primary" | "accent-primary" => Some(ACCENT_PRIMARY),
        "accent_secondary" | "accent-secondary" => Some(ACCENT_SECONDARY),
        "accent_tertiary" | "accent-tertiary" => Some(ACCENT_TERTIARY),
        "accent_amber" | "accent-amber" => Some(ACCENT_AMBER),
        "accent_orange" | "accent-orange" => Some(ACCENT_ORANGE),
        "accent_assistant" | "accent-assistant" => Some(ACCENT_ASSISTANT),
        "error" | "status_error" => Some(STATUS_ERROR),
        "warning" | "status_warning" => Some(STATUS_WARNING),
        "success" | "status_success" => Some(STATUS_SUCCESS),
        "info" | "status_info" => Some(STATUS_INFO),
        _ => None,
    }
}

/// Get a style by semantic name
pub fn get_style(semantic: &str) -> Option<Style> {
    match semantic.to_lowercase().as_str() {
        "default" => Some(style_default()),
        "secondary" | "dim" => Some(style_secondary()),
        "tertiary" | "hint" => Some(style_tertiary()),
        "accent" => Some(style_accent()),
        "error" => Some(style_error()),
        "warning" => Some(style_warning()),
        "success" => Some(style_success()),
        "info" => Some(style_info()),
        "code" => Some(style_code()),
        "user" | "user_message" => Some(style_user_message()),
        "assistant" | "assistant_message" => Some(style_assistant_message()),
        "tool_call" => Some(style_tool_call()),
        "tool_result_success" => Some(style_tool_result_success()),
        "tool_result_error" => Some(style_tool_result_error()),
        "header" => Some(style_header()),
        "footer" => Some(style_footer()),
        "selected" => Some(style_selected()),
        "input" => Some(style_input()),
        "status_running" => Some(style_status_running()),
        "status_idle" => Some(style_status_idle()),
        _ => None,
    }
}
