//! Telegram bot integration.
//!
//! Uses long-polling (no webhook exposure). Access controlled via chat_id allowlist.

use crate::config::TelegramConfig;

/// Telegram bot state and configuration.
pub struct TelegramBot {
    pub bot_token: String,
    pub allowed_chat_ids: Vec<i64>,
}

impl TelegramBot {
    pub fn from_config(config: &TelegramConfig) -> Option<Self> {
        let bot_token = std::env::var(&config.bot_token_env).ok()?;
        Some(Self {
            bot_token,
            allowed_chat_ids: config.allowed_chat_ids.clone(),
        })
    }

    /// Check if a chat ID is allowed to interact with the bot.
    pub fn is_allowed(&self, chat_id: i64) -> bool {
        // If allowlist is empty, allow all (useful for initial setup)
        self.allowed_chat_ids.is_empty() || self.allowed_chat_ids.contains(&chat_id)
    }
}

// TODO: Implement Telegram bot loop using teloxide or raw HTTP API
// The implementation would:
// 1. Start long-polling via getUpdates
// 2. For each incoming message:
//    a. Check chat_id against allowlist
//    b. Route to agent loop with session key = chat_id
//    c. Send response back via sendMessage
// 3. Support /start, /clear, /tools commands

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_allowlist() {
        let bot = TelegramBot {
            bot_token: "test".into(),
            allowed_chat_ids: vec![123, 456],
        };
        assert!(bot.is_allowed(123));
        assert!(!bot.is_allowed(789));
    }

    #[test]
    fn test_telegram_empty_allowlist_allows_all() {
        let bot = TelegramBot {
            bot_token: "test".into(),
            allowed_chat_ids: vec![],
        };
        assert!(bot.is_allowed(123));
        assert!(bot.is_allowed(999));
    }
}
