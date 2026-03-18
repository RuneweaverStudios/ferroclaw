use thiserror::Error;

#[derive(Error, Debug)]
pub enum FerroError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Capability denied: tool '{tool}' requires {required}, session has {available}")]
    CapabilityDenied {
        tool: String,
        required: String,
        available: String,
    },

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Budget exhausted: used {used} of {limit} tokens")]
    BudgetExhausted { used: u64, limit: u64 },

    #[error("Max iterations reached: {0}")]
    MaxIterations(u32),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Channel closed")]
    ChannelClosed,
}

pub type Result<T> = std::result::Result<T, FerroError>;
