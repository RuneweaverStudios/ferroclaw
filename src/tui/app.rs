//! Application state for the Ferroclaw TUI.

/// A single entry in the chat history panel.
#[derive(Debug, Clone)]
pub enum ChatEntry {
    /// A message typed by the user.
    UserMessage(String),
    /// A response from the assistant.
    AssistantMessage(String),
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
    /// Status message.
    pub status: String,
    /// Capabilities display string.
    pub capabilities_str: String,
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
            status: "Ready".into(),
            capabilities_str: String::new(),
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

    /// Scroll chat history up by `n` lines.
    pub fn scroll_up(&mut self, n: u16) {
        let max_scroll = self.total_chat_lines.saturating_sub(self.visible_chat_height);
        self.scroll_offset = (self.scroll_offset + n).min(max_scroll);
    }

    /// Scroll chat history down by `n` lines.
    pub fn scroll_down(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Scroll to the bottom of the chat.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Clear chat history.
    pub fn clear_chat(&mut self) {
        self.chat_history.clear();
        self.scroll_offset = 0;
        self.chat_history.push(ChatEntry::SystemInfo(
            "Chat cleared.".into(),
        ));
    }

    /// Set the status message.
    pub fn set_status(&mut self, status: &str) {
        self.status = status.into();
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
        app.chat_history.push(ChatEntry::UserMessage("hello".into()));
        app.clear_chat();
        assert_eq!(app.chat_history.len(), 1);
        assert!(matches!(&app.chat_history[0], ChatEntry::SystemInfo(_)));
    }
}
