//! Hermes-style TUI module for Ferroclaw.
//!
//! Provides a chat interface similar to the Hermes agent TUI with:
//! - Dark theme
//! - Message bubbles (assistant: "Ferroclaw" header + text; user: orange dot + text)
//! - Bottom status bar with model/process info
//! - Left sidebar with task management

#[path = "hermes_ui.rs"]
mod hermes_ui;

use super::app::{App, ChatEntry};
use super::events::{Event, EventHandler};
use hermes_ui::draw as draw_hermes;

use crate::agent::AgentLoop;
use crate::agent::r#loop::AgentEvent;
use crate::config::Config;
use crate::types::Message;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct ExternalSkill {
    name: String,
    path: PathBuf,
    content: String,
}

type SkillCatalog = BTreeMap<String, ExternalSkill>;

enum SlashAction {
    Continue,
    Send(String),
}

fn discover_external_skills() -> SkillCatalog {
    let mut out = BTreeMap::new();
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let cwd = std::env::current_dir().ok();

    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(home) = &home {
        roots.push(home.join(".hermes/skills"));
        roots.push(home.join(".claude/workspace/skills"));
        roots.push(home.join(".claude/skills"));
        roots.push(home.join(".openclaw"));
        roots.push(home.join(".openclaw/skills"));
    }
    if let Some(cwd) = &cwd {
        roots.push(cwd.join(".claude/workspace/skills"));
        roots.push(cwd.join(".claude/skills"));
        roots.push(cwd.join(".openclaw"));
        roots.push(cwd.join(".openclaw/skills"));
        roots.push(cwd.join("skills"));
    }

    for root in roots {
        scan_skill_md(&root, &mut out);
    }
    out
}

fn scan_skill_md(root: &Path, out: &mut SkillCatalog) {
    if !root.exists() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("SKILL.md"))
                .unwrap_or(false)
            {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                let name = skill_name_from_path_or_frontmatter(&path, &content);
                out.insert(
                    name.clone(),
                    ExternalSkill {
                        name,
                        path: path.clone(),
                        content,
                    },
                );
            }
        }
    }
}

fn skill_name_from_path_or_frontmatter(path: &Path, content: &str) -> String {
    for line in content.lines().take(40) {
        let trimmed = line.trim();
        if let Some(v) = trimmed.strip_prefix("name:") {
            let candidate = v.trim().trim_matches('"').trim_matches('\'');
            if !candidate.is_empty() {
                return candidate.to_string();
            }
        }
    }
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("skill")
        .to_string()
}

fn handle_slash_command(
    raw: &str,
    app: &mut App,
    catalog: &mut SkillCatalog,
    active_skills: &mut BTreeMap<String, ExternalSkill>,
) -> SlashAction {
    let trimmed = raw.trim();
    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next().unwrap_or("");

    match cmd {
        "/help" | "/?" => {
            app.chat_history.push(ChatEntry::SystemInfo(
                "Slash commands: /skills, /skills rescan, /use <skill>, /unuse <skill|all>, /active-skills".into(),
            ));
            SlashAction::Continue
        }
        "/skills" => {
            if matches!(parts.next(), Some("rescan")) {
                *catalog = discover_external_skills();
                app.chat_history.push(ChatEntry::SystemInfo(format!(
                    "Rescanned skills: found {} SKILL.md files.",
                    catalog.len()
                )));
                return SlashAction::Continue;
            }
            if catalog.is_empty() {
                app.chat_history.push(ChatEntry::SystemInfo(
                    "No SKILL.md files found in known locations.".into(),
                ));
                return SlashAction::Continue;
            }
            let mut preview = String::from("Discovered skills:\n");
            for (i, skill) in catalog.values().take(60).enumerate() {
                preview.push_str(&format!("{}. {} ({})\n", i + 1, skill.name, skill.path.display()));
            }
            if catalog.len() > 60 {
                preview.push_str(&format!("... and {} more", catalog.len() - 60));
            }
            app.chat_history.push(ChatEntry::SystemInfo(preview));
            SlashAction::Continue
        }
        "/active-skills" => {
            if active_skills.is_empty() {
                app.chat_history
                    .push(ChatEntry::SystemInfo("No active skills.".into()));
            } else {
                let mut s = String::from("Active skills:\n");
                for skill in active_skills.values() {
                    s.push_str(&format!("- {}\n", skill.name));
                }
                app.chat_history.push(ChatEntry::SystemInfo(s));
            }
            SlashAction::Continue
        }
        "/use" => {
            let target = parts.collect::<Vec<_>>().join(" ");
            if target.is_empty() {
                app.chat_history.push(ChatEntry::Error(
                    "Usage: /use <skill name> (run /skills to list)".into(),
                ));
                return SlashAction::Continue;
            }
            if let Some(skill) = catalog.get(&target).cloned() {
                active_skills.insert(skill.name.clone(), skill.clone());
                app.chat_history.push(ChatEntry::SystemInfo(format!(
                    "Activated skill: {}",
                    skill.name
                )));
            } else {
                app.chat_history.push(ChatEntry::Error(format!(
                    "Skill '{}' not found. Use /skills or /skills rescan.",
                    target
                )));
            }
            SlashAction::Continue
        }
        "/unuse" => {
            let target = parts.collect::<Vec<_>>().join(" ");
            if target.eq_ignore_ascii_case("all") {
                active_skills.clear();
                app.chat_history
                    .push(ChatEntry::SystemInfo("Cleared all active skills.".into()));
            } else if target.is_empty() {
                app.chat_history
                    .push(ChatEntry::Error("Usage: /unuse <skill|all>".into()));
            } else if active_skills.remove(&target).is_some() {
                app.chat_history
                    .push(ChatEntry::SystemInfo(format!("Deactivated skill: {target}")));
            } else {
                app.chat_history
                    .push(ChatEntry::Error(format!("Skill not active: {target}")));
            }
            SlashAction::Continue
        }
        _ => {
            let mut final_input = raw.to_string();
            if !active_skills.is_empty() {
                let mut preface = String::from(
                    "Active skill context (follow as guidance):\n",
                );
                for skill in active_skills.values() {
                    preface.push_str(&format!("\n### SKILL: {}\n", skill.name));
                    // guard against runaway prompt bloat
                    let clipped: String = skill.content.chars().take(5000).collect();
                    preface.push_str(&clipped);
                    preface.push('\n');
                }
                preface.push_str("\n### USER REQUEST\n");
                preface.push_str(raw);
                final_input = preface;
            }
            SlashAction::Send(final_input)
        }
    }
}

/// Run the Hermes-style TUI REPL. Takes ownership of the agent loop and config.
pub async fn run_hermes_tui(mut agent_loop: AgentLoop, config: &Config) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let model_name = config.agent.default_model.clone();
    let token_budget = config.agent.token_budget;

    let mut app = App::new(model_name, token_budget);
    let event_handler = EventHandler::new(250);
    let mut history: Vec<Message> = Vec::new();
    let mut skill_catalog = discover_external_skills();
    let mut active_skills: BTreeMap<String, ExternalSkill> = BTreeMap::new();

    // Add Ferroclaw greeting
    app.chat_history.push(ChatEntry::AssistantMessage(
        "Hello! I'm Ferroclaw, your security-first AI assistant. How can I help you today?".into(),
    ));
    app.chat_history.push(ChatEntry::SystemInfo(
        "Scroll: mouse wheel, PgUp/PgDn, Shift+↑/Shift+↓, or plain ↑/↓ when input is empty. Ctrl+Home/Ctrl+End jump top/bottom.".into(),
    ));
    app.chat_history.push(ChatEntry::SystemInfo(format!(
        "Slash commands enabled: /skills, /skills rescan, /use <skill>, /unuse <skill|all>, /active-skills ({} discovered)",
        skill_catalog.len()
    )));

    // Add some sample tasks to demonstrate the sidebar
    app.add_task(
        "Review security logs".to_string(),
        "Check for unusual access patterns".to_string(),
    );
    app.add_task(
        "Update dependencies".to_string(),
        "Run cargo update and review changes".to_string(),
    );
    app.add_task(
        "Write documentation".to_string(),
        "Document the new API endpoints".to_string(),
    );

    // Main loop
    let result = run_loop(
        &mut terminal,
        &mut app,
        &event_handler,
        &mut agent_loop,
        &mut history,
        &mut skill_catalog,
        &mut active_skills,
    )
    .await;

    // Restore terminal (always, even on error)
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_handler: &EventHandler,
    agent_loop: &mut AgentLoop,
    history: &mut Vec<Message>,
    skill_catalog: &mut SkillCatalog,
    active_skills: &mut BTreeMap<String, ExternalSkill>,
) -> anyhow::Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| draw_hermes(frame, app))?;

        // Handle events
        match event_handler.next()? {
            Event::Tick => {
                // Nothing to do on tick, just redraw
            }
            Event::MouseScrollUp => {
                app.scroll_up(3);
            }
            Event::MouseScrollDown => {
                app.scroll_down(3);
            }
            Event::Key(key_event) => {
                use crossterm::event::KeyCode;
                use crossterm::event::KeyModifiers;

                let code = key_event.code;
                let modifiers = key_event.modifiers;

                // Task management disabled - shortcuts removed
                // if let Some(task_cmd) = Event::Key(key_event).as_task_command() {
                //     handle_task_command(app, task_cmd);
                //     continue;
                // }

                // Ctrl+C: quit
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    return Ok(());
                }

                // Ctrl+L: clear chat
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('l') {
                    app.clear_chat();
                    continue;
                }

                // PageUp / PageDown: scroll chat
                if code == KeyCode::PageUp {
                    app.scroll_up(10);
                    continue;
                }
                if code == KeyCode::PageDown {
                    app.scroll_down(10);
                    continue;
                }

                // Ctrl+Home / Ctrl+End: jump to top/bottom
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Home {
                    app.scroll_to_top();
                    continue;
                }
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::End {
                    app.scroll_to_bottom();
                    continue;
                }

                // Shift+Up / Shift+Down: scroll by 1
                if modifiers.contains(KeyModifiers::SHIFT) && code == KeyCode::Up {
                    app.scroll_up(1);
                    continue;
                }
                if modifiers.contains(KeyModifiers::SHIFT) && code == KeyCode::Down {
                    app.scroll_down(1);
                    continue;
                }

                // Enter: send message (Shift+Enter for newline)
                if code == KeyCode::Enter && !modifiers.contains(KeyModifiers::SHIFT) {
                    let input = app.take_input();
                    if input.is_empty() {
                        continue;
                    }

                    app.chat_history.push(ChatEntry::UserMessage(input.clone()));
                    app.scroll_to_bottom();

                    match handle_slash_command(&input, app, skill_catalog, active_skills) {
                        SlashAction::Continue => {
                            app.set_status("Ready");
                            app.scroll_to_bottom();
                            continue;
                        }
                        SlashAction::Send(effective_input) => {
                            app.set_status("Thinking...");
                            app.iteration = 0;

                            // Redraw with the user message visible
                            terminal.draw(|frame| draw_hermes(frame, app))?;

                            // Run the agent
                            match agent_loop.run(&effective_input, history).await {
                                Ok((response, events)) => {
                                    process_agent_events(app, &events);
                                    app.chat_history.push(ChatEntry::AssistantMessage(response));
                                    app.set_status("Ready");
                                }
                                Err(e) => {
                                    app.chat_history.push(ChatEntry::Error(format!("{e}")));
                                    app.set_status("Error");
                                }
                            }
                            app.scroll_to_bottom();
                            continue;
                        }
                    }
                }

                // Shift+Enter or Alt+Enter: newline in input
                if code == KeyCode::Enter && modifiers.contains(KeyModifiers::SHIFT) {
                    app.input_newline();
                    continue;
                }

                // Backspace
                if code == KeyCode::Backspace {
                    app.input_backspace();
                    continue;
                }

                // Delete
                if code == KeyCode::Delete {
                    app.input_delete();
                    continue;
                }

                // Arrow keys for cursor movement in input (only if not task navigation)
                if code == KeyCode::Left {
                    app.input_move_left();
                    continue;
                }
                if code == KeyCode::Right {
                    app.input_move_right();
                    continue;
                }
                if code == KeyCode::Up && !modifiers.contains(KeyModifiers::SHIFT) {
                    if app.input_is_blank() {
                        app.scroll_up(1);
                    } else {
                        app.input_move_up();
                    }
                    continue;
                }
                if code == KeyCode::Down && !modifiers.contains(KeyModifiers::SHIFT) {
                    if app.input_is_blank() {
                        app.scroll_down(1);
                    } else {
                        app.input_move_down();
                    }
                    continue;
                }

                // Home / End
                if code == KeyCode::Home {
                    app.input_home();
                    continue;
                }
                if code == KeyCode::End {
                    app.input_end();
                    continue;
                }

                // Character input
                if let KeyCode::Char(c) = code {
                    app.input_char(c);
                }

                // Tab -> 4 spaces
                if code == KeyCode::Tab {
                    for _ in 0..4 {
                        app.input_char(' ');
                    }
                }
            }
            Event::Resize(_, _) => {
                // Terminal will redraw on next iteration
            }
        }
    }
}

/// Process AgentEvents into ChatEntry items for the TUI.
fn process_agent_events(app: &mut App, events: &[AgentEvent]) {
    for event in events {
        match event {
            AgentEvent::ToolCallStart { name, .. } => {
                app.chat_history.push(ChatEntry::ToolCall {
                    name: name.clone(),
                    args: String::new(),
                });
                app.iteration += 1;
            }
            AgentEvent::LlmRound { .. }
            | AgentEvent::ModelToolChoice { .. }
            | AgentEvent::ParallelToolBatch { .. } => {}
            AgentEvent::ToolResult {
                name,
                content,
                is_error,
                ..
            } => {
                app.chat_history.push(ChatEntry::ToolResult {
                    name: name.clone(),
                    content: content.clone(),
                    is_error: *is_error,
                });
            }
            AgentEvent::TokenUsage {
                input,
                output,
                total_used,
            } => {
                app.tokens_used = *total_used;
                app.last_input_tokens = *input;
                app.last_output_tokens = *output;
            }
            AgentEvent::Error(msg) => {
                app.chat_history.push(ChatEntry::Error(msg.clone()));
            }
            AgentEvent::TextDelta(_) | AgentEvent::Done { .. } => {
                // Text deltas are already captured in the final response
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_name_prefers_frontmatter_name() {
        let p = PathBuf::from("/tmp/some/path/SKILL.md");
        let c = "---\nname: my-skill\n---\nbody";
        assert_eq!(skill_name_from_path_or_frontmatter(&p, c), "my-skill");
    }

    #[test]
    fn skill_name_falls_back_to_parent_dir() {
        let p = PathBuf::from("/tmp/demo-skill/SKILL.md");
        let c = "# title only";
        assert_eq!(skill_name_from_path_or_frontmatter(&p, c), "demo-skill");
    }
}
