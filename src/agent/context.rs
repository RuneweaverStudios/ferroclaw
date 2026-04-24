//! Context window management: token tracking, pruning, and budget enforcement.

use crate::types::Message;

/// Manages the conversation context within token budget constraints.
pub struct ContextManager {
    pub token_budget: u64,
    pub tokens_used: u64,
    /// Minimum messages to preserve (system + last N user/assistant pairs)
    pub min_preserve: usize,
}

impl ContextManager {
    pub fn new(token_budget: u64) -> Self {
        Self {
            token_budget,
            tokens_used: 0,
            min_preserve: 6,
        }
    }

    /// Estimate total tokens for a message list.
    pub fn estimate_total(messages: &[Message]) -> u64 {
        messages.iter().map(|m| m.estimated_tokens()).sum()
    }

    /// Check if adding a message would exceed the budget.
    pub fn would_exceed(&self, messages: &[Message], new_msg: &Message) -> bool {
        let current = Self::estimate_total(messages);
        let additional = new_msg.estimated_tokens();
        current + additional > self.token_budget
    }

    /// Prune messages to fit within budget. Preserves system messages and
    /// the most recent conversation turns. Removes from the middle.
    pub fn prune_to_fit(&self, messages: &mut Vec<Message>) {
        let total = Self::estimate_total(messages);
        if total <= self.token_budget {
            return;
        }

        let target = (self.token_budget as f64 * 0.8) as u64; // Prune to 80% of budget

        // Separate system messages (always kept) from conversation
        let system_count = messages
            .iter()
            .take_while(|m| m.role == crate::types::Role::System)
            .count();

        // Keep system messages + first 2 user messages + last N messages
        let keep_start = system_count + 2;
        let keep_end = self.min_preserve;

        if messages.len() <= keep_start + keep_end {
            return; // Can't prune further
        }

        // Remove messages from the middle until we're under target
        let mut current_total = total;
        let mut to_remove = Vec::new();

        for i in (keep_start..messages.len().saturating_sub(keep_end)).rev() {
            if current_total <= target {
                break;
            }
            current_total -= messages[i].estimated_tokens();
            to_remove.push(i);
        }

        // Insert a summary marker where we pruned
        if !to_remove.is_empty() {
            let removed_count = to_remove.len();
            for i in to_remove.into_iter() {
                messages.remove(i);
            }

            // Insert a marker so the LLM knows context was pruned
            if keep_start < messages.len() {
                messages.insert(
                    keep_start,
                    Message::system(format!(
                        "[{removed_count} earlier messages pruned to fit context window]"
                    )),
                );
            }
        }
    }

    /// Update token usage tracking from provider response.
    pub fn record_usage(&mut self, input_tokens: u64, output_tokens: u64) {
        self.tokens_used += input_tokens + output_tokens;
    }

    /// Remaining token budget.
    pub fn remaining(&self) -> u64 {
        self.token_budget.saturating_sub(self.tokens_used)
    }

    /// Fraction of budget consumed.
    pub fn usage_fraction(&self) -> f64 {
        if self.token_budget == 0 {
            return 1.0;
        }
        self.tokens_used as f64 / self.token_budget as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;

    #[test]
    fn test_estimate_total() {
        let msgs = vec![
            Message::system("You are helpful."),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];
        let total = ContextManager::estimate_total(&msgs);
        assert!(total > 0);
    }

    #[test]
    fn test_prune_to_fit() {
        let ctx = ContextManager::new(100); // Very small budget

        let mut msgs: Vec<Message> = Vec::new();
        msgs.push(Message::system("System prompt"));
        for i in 0..20 {
            msgs.push(Message::user(format!(
                "Message {i} with some content padding"
            )));
            msgs.push(Message::assistant(format!(
                "Response {i} with more padding text"
            )));
        }

        let original_len = msgs.len();
        ctx.prune_to_fit(&mut msgs);
        assert!(msgs.len() < original_len);
    }

    #[test]
    fn test_remaining_budget() {
        let mut ctx = ContextManager::new(1000);
        assert_eq!(ctx.remaining(), 1000);
        ctx.record_usage(100, 50);
        assert_eq!(ctx.remaining(), 850);
    }
}
