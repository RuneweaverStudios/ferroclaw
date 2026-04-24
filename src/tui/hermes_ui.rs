//! Hermes-style chat TUI implementation for Ferroclaw.
//!
//! This module provides a chat interface similar to the Hermes agent TUI with:
//! - Dark theme
//! - Message bubbles (assistant: "Ferroclaw" header + text; user: orange dot + text)
//! - Bottom status bar with model/process info

use crate::tui::app::{App, ChatEntry};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Draw the entire Hermes-style TUI layout.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Chat history
            Constraint::Length(1), // Status bar (above input)
            Constraint::Length(5), // Input area
        ])
        .split(frame.area());

    draw_chat(frame, app, chunks[0]);
    draw_status_bar(frame, app, chunks[1]);
    draw_input(frame, app, chunks[2]);
}

/// Bottom status bar with model/process info.
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_text = format!(
        " {} | {} | {} | {}s",
        app.model_name,
        format!("{}/{}", app.tokens_used, app.token_budget),
        format!(
            "[{} {}%]",
            if app.iteration > 0 { "▓" } else { " " },
            app.iteration * 10
        ),
        app.iteration,
    );

    let status = Paragraph::new(status_text).style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(status, area);
}

/// Hermes-style chat history with message bubbles.
fn draw_chat(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_width = area.width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = Vec::new();

    for entry in &app.chat_history {
        match entry {
            ChatEntry::TranscriptLine(s) => {
                lines.push(Line::from(Span::styled(
                    s.as_str(),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            ChatEntry::UserMessage(text) => {
                // User message with orange dot indicator
                lines.push(Line::from(vec![
                    Span::styled(
                        "●",
                        Style::default()
                            .fg(Color::Rgb(255, 107, 53))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " You: ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(text),
                ]));
            }
            ChatEntry::AssistantMessage(text) => {
                // Assistant message with "Ferroclaw" header
                lines.push(Line::from(vec![Span::styled(
                    "Ferroclaw: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]));
                for line in text.lines() {
                    lines.push(Line::from(format!("    {line}")));
                }
            }
            ChatEntry::ToolCall { name, args: _ } => {
                lines.push(Line::from(vec![
                    Span::styled("  -> ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        name,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
            ChatEntry::ToolResult {
                name,
                content,
                is_error,
            } => {
                let color = if *is_error {
                    Color::Red
                } else {
                    Color::DarkGray
                };
                let label = if *is_error { "ERR" } else { "OK" };
                lines.push(Line::from(vec![Span::styled(
                    format!("  <- {name} [{label}] "),
                    Style::default().fg(color),
                )]));
                // Show first 3 lines of result, truncated
                for (i, ln) in content.lines().enumerate() {
                    if i >= 3 {
                        lines.push(Line::from(Span::styled(
                            format!("     ... ({} more lines)", content.lines().count() - 3),
                            Style::default().fg(Color::DarkGray),
                        )));
                        break;
                    }
                    let max = inner_width.saturating_sub(8);
                    let truncated = if ln.chars().count() > max {
                        format!("{}...", ln.chars().take(max).collect::<String>())
                    } else {
                        ln.to_string()
                    };
                    lines.push(Line::from(Span::styled(
                        format!("     {truncated}"),
                        Style::default().fg(color),
                    )));
                }
            }
            ChatEntry::SystemInfo(text) => {
                lines.push(Line::from(Span::styled(
                    text,
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::ITALIC),
                )));
            }
            ChatEntry::Error(text) => {
                lines.push(Line::from(vec![
                    Span::styled(
                        "ERROR: ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(text, Style::default().fg(Color::Red)),
                ]));
            }
        }
        lines.push(Line::from("")); // Blank line between entries
    }

    app.total_chat_lines = lines.len() as u16;
    app.visible_chat_height = area.height.saturating_sub(2); // Account for borders

    // Keep newest entries visually anchored near the bottom of the chat box when content is short.
    // This makes new messages appear directly above the input area, Hermes-style.
    let visible = app.visible_chat_height as usize;
    if visible > 0 && lines.len() < visible {
        let mut padded = Vec::with_capacity(visible);
        padded.extend(std::iter::repeat_n(Line::from(""), visible - lines.len()));
        padded.extend(lines);
        lines = padded;
    }

    // Apply scroll offset (scroll_offset = 0 means bottom-most view)
    let total = lines.len() as u16;
    let visible = app.visible_chat_height;
    let scroll = if total > visible {
        let max_offset = total - visible;
        let from_bottom = app.scroll_offset.min(max_offset);
        max_offset - from_bottom
    } else {
        0
    };

    let chat = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Chat ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(chat, area);
}

/// Multiline input area with cursor.
fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input_text: Vec<Line> = app
        .input_lines
        .iter()
        .map(|l: &String| Line::from(l.as_str()))
        .collect();

    let input = Paragraph::new(Text::from(input_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Type your message... ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);

    // Position the cursor
    let cursor_x = area.x + 1 + app.cursor_col as u16;
    let cursor_y = area.y + 1 + app.cursor_line as u16;
    if cursor_x < area.x + area.width - 1 && cursor_y < area.y + area.height - 1 {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
