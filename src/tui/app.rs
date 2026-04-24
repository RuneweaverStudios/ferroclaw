//! Application state for the Ferroclaw TUI.

use std::time::Instant;

/// Status of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

/// A single task in the task list.
#[derive(Debug, Clone)]
pub struct Task {
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: Instant,
}

impl Task {
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description,
            status: TaskStatus::Pending,
            created_at: Instant::now(),
        }
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }
}

/// A single entry in the chat history panel.
#[derive(Debug, Clone)]
pub enum ChatEntry {
    /// A message typed by the user.
    UserMessage(String),
    /// A response from the assistant.
    AssistantMessage(String),
    /// Orchestrator-style transcript line (`>`, `→`, `←`, `◆`, `⋯`, `[…]`).
    TranscriptLine(String),
    /// A tool call made by the agent.
    ToolCall { name: String, args: String },
    /// The result of a tool call.
    ToolResult {
        name: String,
        content: String,
        is_error: bool,
    },
    /// An informational system message.
    SystemInfo(String),
    /// An error message.
    Error(String),
}

/// Holds all TUI state.
pub struct App {
    /// Chat history entries displayed in the main panel.
    pub chat_history: Vec<ChatEntry>,
    /// Input buffer lines (supports multiline editing).
    pub input_lines: Vec<String>,
    /// Cursor position: (line_index, column_index).
    pub cursor_line: usize,
    pub cursor_col: usize,
    /// Scroll offset for the chat panel (0 = bottom-most view).
    pub scroll_offset: u16,
    /// Total rendered lines in chat (updated each frame).
    pub total_chat_lines: u16,
    /// Visible chat area height (updated each frame).
    pub visible_chat_height: u16,
    /// Whether we're in "sticky bottom" mode (auto-scroll on new content).
    pub sticky_bottom: bool,
    /// Model name for the banner.
    pub model_name: String,
    /// Token budget.
    pub token_budget: u64,
    /// Tokens used so far.
    pub tokens_used: u64,
    /// Last request input tokens.
    pub last_input_tokens: u64,
    /// Last request output tokens.
    pub last_output_tokens: u64,
    /// Current agent iteration within a turn.
    pub iteration: u32,
    /// Short status verb for the orchestrator bar (e.g. "Ready", "Thinking…").
    pub verb: String,
    /// True while the agent is running a turn.
    pub is_running: bool,
    /// True when an error has occurred (shows red ● indicator).
    pub is_error: bool,
    /// When the current agent run started (for long-wait nudges and glitter verb time tracking).
    pub run_started_at: Option<Instant>,
    /// Last second bucket we logged a `[Ns] Still waiting` nudge (0 = none).
    pub last_nudge_sec: u32,
    /// Status message.
    pub status: String,
    /// Capabilities display string.
    pub capabilities_str: String,
    /// Task list for the sidebar.
    pub tasks: Vec<Task>,
    /// Index of the currently selected task in the sidebar.
    pub selected_task_index: usize,
    /// Currently active tool names (for glitter verbs).
    pub active_tools: Vec<String>,
}

impl App {
    pub fn new(model_name: String, token_budget: u64) -> Self {
        Self {
            chat_history: Vec::new(),
            input_lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            total_chat_lines: 0,
            visible_chat_height: 0,
            model_name,
            token_budget,
            tokens_used: 0,
            last_input_tokens: 0,
            last_output_tokens: 0,
            iteration: 0,
            verb: "Ready".into(),
            is_running: false,
            is_error: false,
            run_started_at: None,
            last_nudge_sec: 0,
            status: "Ready".into(),
            capabilities_str: String::new(),
            tasks: Vec::new(),
            selected_task_index: 0,
            sticky_bottom: true,
            active_tools: Vec::new(),
        }
    }

    /// Get the current input as a single string.
    pub fn input_text(&self) -> String {
        self.input_lines.join("\n")
    }

    /// Take the input text and reset the input buffer.
    pub fn take_input(&mut self) -> String {
        let text = self.input_text();
        self.input_lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        text.trim().to_string()
    }

    /// Insert a character at the cursor position.
    pub fn input_char(&mut self, c: char) {
        let line = &mut self.input_lines[self.cursor_line];
        let byte_idx = char_to_byte_index(line, self.cursor_col);
        line.insert(byte_idx, c);
        self.cursor_col += 1;
    }

    /// Insert a newline at the cursor position.
    pub fn input_newline(&mut self) {
        let current_line = &self.input_lines[self.cursor_line];
        let byte_idx = char_to_byte_index(current_line, self.cursor_col);
        let rest = current_line[byte_idx..].to_string();
        self.input_lines[self.cursor_line] = current_line[..byte_idx].to_string();
        self.cursor_line += 1;
        self.input_lines.insert(self.cursor_line, rest);
        self.cursor_col = 0;
    }

    /// Handle backspace.
    pub fn input_backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.input_lines[self.cursor_line];
            let byte_idx = char_to_byte_index(line, self.cursor_col - 1);
            let next_byte_idx = char_to_byte_index(line, self.cursor_col);
            line.drain(byte_idx..next_byte_idx);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            // Merge with previous line
            let current = self.input_lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.input_lines[self.cursor_line].chars().count();
            self.input_lines[self.cursor_line].push_str(&current);
        }
    }

    /// Handle delete key.
    pub fn input_delete(&mut self) {
        let line_char_count = self.input_lines[self.cursor_line].chars().count();
        if self.cursor_col < line_char_count {
            let line = &mut self.input_lines[self.cursor_line];
            let byte_idx = char_to_byte_index(line, self.cursor_col);
            let next_byte_idx = char_to_byte_index(line, self.cursor_col + 1);
            line.drain(byte_idx..next_byte_idx);
        } else if self.cursor_line + 1 < self.input_lines.len() {
            // Merge next line into current
            let next = self.input_lines.remove(self.cursor_line + 1);
            self.input_lines[self.cursor_line].push_str(&next);
        }
    }

    /// Move cursor left.
    pub fn input_move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.input_lines[self.cursor_line].chars().count();
        }
    }

    /// Move cursor right.
    pub fn input_move_right(&mut self) {
        let line_len = self.input_lines[self.cursor_line].chars().count();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.input_lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up within input.
    pub fn input_move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let line_len = self.input_lines[self.cursor_line].chars().count();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    /// Move cursor down within input.
    pub fn input_move_down(&mut self) {
        if self.cursor_line + 1 < self.input_lines.len() {
            self.cursor_line += 1;
            let line_len = self.input_lines[self.cursor_line].chars().count();
            self.cursor_col = self.cursor_col.min(line_len);
        }
    }

    /// Move cursor to start of line.
    pub fn input_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line.
    pub fn input_end(&mut self) {
        self.cursor_col = self.input_lines[self.cursor_line].chars().count();
    }

    /// Returns true when composer/input is effectively empty.
    pub fn input_is_blank(&self) -> bool {
        self.input_lines.len() == 1
            && self.input_lines[0].is_empty()
            && self.cursor_line == 0
            && self.cursor_col == 0
    }

    /// Scroll chat history up by `n` lines (show older content).
    pub fn scroll_up(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(n);
        self.clamp_scroll();
        // User manually scrolled up, disable sticky mode
        if n > 0 {
            self.sticky_bottom = false;
        }
    }

    /// Scroll chat history down by `n` lines (show newer content).
    pub fn scroll_down(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
        // If we scrolled all the way to bottom, re-enable sticky mode
        if self.scroll_offset == 0 {
            self.sticky_bottom = true;
        }
    }

    /// Scroll to the bottom of the chat (show newest content).
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.sticky_bottom = true;
    }

    /// Scroll to the top of the chat (show oldest content).
    pub fn scroll_to_top(&mut self) {
        let max_scroll = self
            .total_chat_lines
            .saturating_sub(self.visible_chat_height);
        self.scroll_offset = max_scroll;
        self.sticky_bottom = false;
    }

    /// Ensure scroll_offset is within valid bounds.
    fn clamp_scroll(&mut self) {
        let max_scroll = self
            .total_chat_lines
            .saturating_sub(self.visible_chat_height);
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    /// Check if the chat is currently scrolled to the bottom.
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Auto-scroll to bottom if sticky mode is enabled (for new content).
    pub fn auto_scroll_if_sticky(&mut self) {
        if self.sticky_bottom {
            self.scroll_offset = 0;
        }
    }

    /// Clear chat history.
    pub fn clear_chat(&mut self) {
        self.chat_history.clear();
        self.scroll_offset = 0;
        self.verb = "Ready".into();
        self.is_running = false;
        self.is_error = false;
        self.run_started_at = None;
        self.last_nudge_sec = 0;
        self.active_tools.clear();
        self.chat_history
            .push(ChatEntry::SystemInfo("Chat cleared.".into()));
    }

    /// Set the status message.
    pub fn set_status(&mut self, status: &str) {
        self.status = status.into();
    }

    // ===== Task Management Methods =====

    /// Add a new task to the task list.
    pub fn add_task(&mut self, title: String, description: String) {
        let task = Task::new(title, description);
        self.tasks.push(task);
        // Auto-select the new task
        if !self.tasks.is_empty() {
            self.selected_task_index = self.tasks.len() - 1;
        }
    }

    /// Remove the currently selected task.
    pub fn remove_selected_task(&mut self) {
        if !self.tasks.is_empty() {
            self.tasks.remove(self.selected_task_index);
            // Adjust selection index
            if self.tasks.is_empty() {
                self.selected_task_index = 0;
            } else if self.selected_task_index >= self.tasks.len() {
                self.selected_task_index = self.tasks.len() - 1;
            }
        }
    }

    /// Toggle the status of the currently selected task.
    pub fn toggle_task_status(&mut self) {
        if !self.tasks.is_empty() {
            let task = &mut self.tasks[self.selected_task_index];
            task.status = match task.status {
                TaskStatus::Pending => TaskStatus::InProgress,
                TaskStatus::InProgress => TaskStatus::Completed,
                TaskStatus::Completed => TaskStatus::Pending,
            };
        }
    }

    /// Move selection up in the task list.
    pub fn task_select_up(&mut self) {
        if self.selected_task_index > 0 {
            self.selected_task_index -= 1;
        }
    }

    /// Move selection down in the task list.
    pub fn task_select_down(&mut self) {
        if self.selected_task_index + 1 < self.tasks.len() {
            self.selected_task_index += 1;
        }
    }

    /// Get the currently selected task.
    pub fn selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_task_index)
    }

    // ===== Glitter Verbs Integration =====

    /// Add a tool to the active tools list.
    pub fn add_active_tool(&mut self, name: String) {
        if !name.is_empty() && !self.active_tools.contains(&name) {
            self.active_tools.push(name);
        }
    }

    /// Remove a tool from the active tools list.
    pub fn remove_active_tool(&mut self, name: &str) {
        self.active_tools.retain(|n| n != name);
    }

    /// Clear all active tools.
    pub fn clear_active_tools(&mut self) {
        self.active_tools.clear();
    }

    /// Update the verb based on current state using glitter verbs.
    pub fn update_glitter_verb(&mut self) {
        use crate::tui::glitter_verbs::*;

        self.verb = if !self.is_running {
            "ready".to_string()
        } else if !self.active_tools.is_empty() {
            glitter_verb_for_tools(&self.active_tools)
        } else {
            let elapsed = elapsed_ms_since(self.run_started_at);
            glitter_verb_for_llm_pending(elapsed, self.iteration)
        };
    }
}

/// Convert a char-based column index to a byte index in a string.
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_char_and_take() {
        let mut app = App::new("test-model".into(), 100_000);
        app.input_char('h');
        app.input_char('i');
        assert_eq!(app.input_text(), "hi");
        let taken = app.take_input();
        assert_eq!(taken, "hi");
        assert_eq!(app.input_text(), "");
    }

    #[test]
    fn test_input_backspace() {
        let mut app = App::new("test-model".into(), 100_000);
        app.input_char('a');
        app.input_char('b');
        app.input_char('c');
        app.input_backspace();
        assert_eq!(app.input_text(), "ab");
    }

    #[test]
    fn test_input_newline() {
        let mut app = App::new("test-model".into(), 100_000);
        app.input_char('a');
        app.input_newline();
        app.input_char('b');
        assert_eq!(app.input_text(), "a\nb");
        assert_eq!(app.input_lines.len(), 2);
    }

    #[test]
    fn test_scroll() {
        let mut app = App::new("test-model".into(), 100_000);
        app.total_chat_lines = 100;
        app.visible_chat_height = 20;
        app.scroll_up(5);
        assert_eq!(app.scroll_offset, 5);
        app.scroll_down(3);
        assert_eq!(app.scroll_offset, 2);
        app.scroll_to_bottom();
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_clear_chat() {
        let mut app = App::new("test-model".into(), 100_000);
        app.chat_history
            .push(ChatEntry::UserMessage("hello".into()));
        app.clear_chat();
        assert_eq!(app.chat_history.len(), 1);
        assert!(matches!(&app.chat_history[0], ChatEntry::SystemInfo(_)));
        // Verify error state is cleared
        assert!(!app.is_error);
    }

    #[test]
    fn test_task_management() {
        let mut app = App::new("test-model".into(), 100_000);

        // Add tasks
        app.add_task("Task 1".to_string(), "Description 1".to_string());
        app.add_task("Task 2".to_string(), "Description 2".to_string());

        assert_eq!(app.tasks.len(), 2);
        assert_eq!(app.selected_task_index, 1); // Last task selected

        // Test selection movement
        app.task_select_up();
        assert_eq!(app.selected_task_index, 0);
        app.task_select_down();
        assert_eq!(app.selected_task_index, 1);

        // Test status toggle
        app.toggle_task_status();
        assert_eq!(app.tasks[1].status, TaskStatus::InProgress);
        app.toggle_task_status();
        assert_eq!(app.tasks[1].status, TaskStatus::Completed);
        app.toggle_task_status();
        assert_eq!(app.tasks[1].status, TaskStatus::Pending);

        // Test task removal
        app.remove_selected_task();
        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.selected_task_index, 0);
    }

    #[test]
    fn test_selected_task() {
        let mut app = App::new("test-model".into(), 100_000);

        assert!(app.selected_task().is_none());

        app.add_task("Test Task".to_string(), "Test Description".to_string());

        let task = app.selected_task();
        assert!(task.is_some());
        assert_eq!(task.unwrap().title, "Test Task");
    }

    #[test]
    fn test_thinking_indicator_states() {
        let mut app = App::new("test-model".into(), 100_000);

        // Initial state: ready (○ Green)
        assert!(!app.is_running);
        assert!(!app.is_error);
        assert_eq!(app.verb, "Ready");

        // Running state: processing (● Cyan)
        app.is_running = true;
        assert!(app.is_running);
        assert!(!app.is_error);

        // Error state: error (● Red)
        app.is_running = false;
        app.is_error = true;
        app.verb = "error".to_string();
        assert!(!app.is_running);
        assert!(app.is_error);
        assert_eq!(app.verb, "error");

        // Clear error on new message
        app.is_error = false;
        assert!(!app.is_error);
    }

    #[test]
    fn test_clear_chat_resets_error_state() {
        let mut app = App::new("test-model".into(), 100_000);

        // Set error state
        app.is_error = true;
        app.verb = "error".to_string();
        assert!(app.is_error);

        // Clear chat should reset error state
        app.clear_chat();
        assert!(!app.is_error);
        assert_eq!(app.verb, "Ready");
    }

    #[test]
    fn test_active_tools_management() {
        let mut app = App::new("test-model".into(), 100_000);

        // Add tools
        app.add_active_tool("read_file".to_string());
        app.add_active_tool("write_file".to_string());
        assert_eq!(app.active_tools.len(), 2);

        // Don't add duplicate
        app.add_active_tool("read_file".to_string());
        assert_eq!(app.active_tools.len(), 2);

        // Remove tool
        app.remove_active_tool("read_file");
        assert_eq!(app.active_tools.len(), 1);
        assert_eq!(app.active_tools[0], "write_file");

        // Clear all tools
        app.clear_active_tools();
        assert_eq!(app.active_tools.len(), 0);
    }

    #[test]
    fn test_update_glitter_verb() {
        let mut app = App::new("test-model".into(), 100_000);

        // Initial state: not running
        app.update_glitter_verb();
        assert_eq!(app.verb, "ready");

        // Running with active tool
        app.is_running = true;
        app.add_active_tool("read_file".to_string());
        app.update_glitter_verb();
        assert!(app.verb.contains("Reading"));

        // Running without active tools (LLM pending)
        app.clear_active_tools();
        app.run_started_at = Some(Instant::now());
        app.update_glitter_verb();
        // Should be one of the LLM pending verbs
        assert!(app.verb.contains("…"));
        assert!(!app.verb.contains("Reading"));
    }
}
