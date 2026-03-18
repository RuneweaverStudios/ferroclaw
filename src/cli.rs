//! CLI subcommands for ferroclaw.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ferroclaw")]
#[command(about = "Security-first AI agent with native MCP and DietMCP compression")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive onboarding wizard — configure providers, skills, channels
    Setup,

    /// Start interactive REPL
    Run {
        /// Disable the TUI and use a basic text REPL instead
        #[arg(long)]
        no_tui: bool,
    },

    /// Execute a single prompt and exit
    Exec {
        /// The prompt to execute
        prompt: String,
    },

    /// MCP server management
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Start HTTP gateway and messaging bots
    Serve,

    /// Verify audit log integrity
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },
}

#[derive(Subcommand)]
pub enum McpCommands {
    /// List configured MCP servers and their tools
    List {
        /// Specific server to list
        server: Option<String>,
        /// Force refresh (bypass cache)
        #[arg(long)]
        refresh: bool,
    },
    /// Show diet skill summaries (compressed tool descriptions)
    Diet {
        /// Specific server
        server: Option<String>,
    },
    /// Execute a tool on an MCP server
    Exec {
        /// Server name
        server: String,
        /// Tool name
        tool: String,
        /// JSON arguments
        #[arg(long)]
        args: String,
        /// Output format: summary, minified, csv
        #[arg(long, default_value = "summary")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize a new config file
    Init,
    /// Show current configuration
    Show,
    /// Print config file path
    Path,
}

#[derive(Subcommand)]
pub enum AuditCommands {
    /// Verify the integrity of the audit log
    Verify,
    /// Show audit log path
    Path,
}
