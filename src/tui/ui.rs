//! UI rendering for the Ferroclaw TUI using ratatui.

use super::app::{App, ChatEntry};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

/// Draw the entire TUI layout.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Banner bar
            Constraint::Min(5),    // Chat history
            Constraint::Length(5), // Input area
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    draw_banner(frame, app, chunks[0]);
    draw_chat(frame, app, chunks[1]);
    draw_input(frame, app, chunks[2]);
    draw_status(frame, app, chunks[3]);
}

/// Top banner bar: model name, token usage, iteration count.
fn draw_banner(frame: &mut Frame, app: &App, area: Rect) {
    let budget_pct = if app.token_budget > 0 {
        (app.tokens_used as f64 / app.token_budget as f64 * 100.0) as u64
    } else {
        0
    };

    let banner_text = format!(
        " ferroclaw | model: {} | tokens: {}/{} ({}%) | iter: {}",
        app.model_name, app.tokens_used, app.token_budget, budget_pct, app.iteration,
    );

    let banner = Paragraph::new(banner_text).style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(banner, area);
}

/// Scrollable chat history panel.
fn draw_chat(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = Vec::new();

    for entry in &app.chat_history {
        match entry {
            ChatEntry::UserMessage(text) => {
                lines.push(Line::from(vec![
                    Span::styled("You: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(text),
                ]));
            }
            ChatEntry::AssistantMessage(text) => {
                lines.push(Line::from(vec![
                    Span::styled("AI: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
                for line in text.lines() {
                    lines.push(Line::from(format!("    {line}")));
                }
            }
            ChatEntry::ToolCall { name, args: _ } => {
                lines.push(Line::from(vec![
                    Span::styled("  -> ", Style::default().fg(Color::Yellow)),
                    Span::styled(name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]));
            }
            ChatEntry::ToolResult { name, content, is_error } => {
                let color = if *is_error { Color::Red } else { Color::DarkGray };
                let label = if *is_error { "ERR" } else { "OK" };
                lines.push(Line::from(vec![
                    Span::styled(format!("  <- {name} [{label}] "), Style::default().fg(color)),
                ]));
                // Show first 3 lines of result, truncated
                for (i, line) in content.lines().enumerate() {
                    if i >= 3 {
                        lines.push(Line::from(Span::styled(
                            format!("     ... ({} more lines)", content.lines().count() - 3),
                            Style::default().fg(Color::DarkGray),
                        )));
                        break;
                    }
                    let truncated = if line.len() > inner_width.saturating_sub(5) {
                        format!("{}...", &line[..inner_width.saturating_sub(8)])
                    } else {
                        line.to_string()
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
                    Style::default().fg(Color::Blue).add_modifier(Modifier::ITALIC),
                )));
            }
            ChatEntry::Error(text) => {
                lines.push(Line::from(vec![
                    Span::styled("ERROR: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(text, Style::default().fg(Color::Red)),
                ]));
            }
        }
        lines.push(Line::from("")); // Blank line between entries
    }

    app.total_chat_lines = lines.len() as u16;
    app.visible_chat_height = area.height.saturating_sub(2); // Account for borders

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
                .title(" Chat "),
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
        .map(|l| Line::from(l.as_str()))
        .collect();

    let input = Paragraph::new(Text::from(input_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input (Enter to send, Shift+Enter for newline) ")
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

/// Bottom status bar.
fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let last_usage = if app.last_input_tokens > 0 || app.last_output_tokens > 0 {
        format!(
            " | last: in={} out={}",
            app.last_input_tokens, app.last_output_tokens
        )
    } else {
        String::new()
    };

    let status_text = format!(
        " {} | Ctrl+C: quit | Ctrl+L: clear | PgUp/PgDn: scroll{}",
        app.status, last_usage,
    );

    let status = Paragraph::new(status_text).style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White),
    );
    frame.render_widget(status, area);
}
