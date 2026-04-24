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
        /// Emit machine-readable benchmark telemetry footer for harness adapters.
        #[arg(long)]
        benchmark_json: bool,
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

    /// Task management
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    /// Plan mode for structured multi-phase planning
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
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

#[derive(Subcommand)]
pub enum TaskCommands {
    /// Create a new task
    Create {
        /// Brief title for the task
        #[arg(long)]
        subject: String,
        /// Detailed description of what needs to be done
        #[arg(long)]
        description: String,
        /// Present continuous form shown in spinner (e.g., "Running tests")
        #[arg(long)]
        active_form: Option<String>,
        /// Task owner
        #[arg(long)]
        owner: Option<String>,
    },

    /// List tasks with optional filtering
    List {
        /// Filter by status (pending, in_progress, completed)
        #[arg(long)]
        status: Option<String>,
        /// Filter by owner
        #[arg(long)]
        owner: Option<String>,
    },

    /// Show task details
    Show {
        /// Task ID
        id: String,
    },

    /// Update task status
    Update {
        /// Task ID
        id: String,
        /// New status (pending, in_progress, completed)
        #[arg(long)]
        status: String,
        /// New subject
        #[arg(long)]
        subject: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
    },

    /// Delete a task
    Delete {
        /// Task ID
        id: String,
    },

    /// Add dependency (task blocks another task)
    AddBlock {
        /// Task ID that will block
        id: String,
        /// Task ID to be blocked
        blocks_id: String,
    },

    /// Remove dependency
    RemoveBlock {
        /// Task ID
        id: String,
        /// Task ID to no longer block
        blocks_id: String,
    },

    /// Show tasks that are blocking this task
    Blocking {
        /// Task ID
        id: String,
    },

    /// Show tasks that this task is blocking
    Blocked {
        /// Task ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum PlanCommands {
    /// Initialize a new plan
    Init {
        /// Plan description
        description: Option<String>,
    },

    /// Show current plan status
    Status,

    /// Create a new plan step
    CreateStep {
        /// Brief title for the step
        #[arg(long)]
        subject: String,
        /// Detailed description of what needs to be done
        #[arg(long)]
        description: String,
        /// Present continuous form shown in spinner (e.g., "Running tests")
        #[arg(long)]
        active_form: Option<String>,
        /// Comma-separated acceptance criteria
        #[arg(long)]
        acceptance_criteria: Option<String>,
        /// Comma-separated step IDs this step depends on
        #[arg(long)]
        depends_on: Option<String>,
        /// Whether this step requires approval before starting
        #[arg(long)]
        requires_approval: bool,
    },

    /// List all steps in the plan
    ListSteps,

    /// Show step details
    ShowStep {
        /// Step ID
        id: String,
    },

    /// Update step status
    UpdateStep {
        /// Step ID
        id: String,
        /// New status (pending, in_progress, completed, failed)
        #[arg(long)]
        status: String,
    },

    /// Approve a step that requires approval
    ApproveStep {
        /// Step ID
        id: String,
    },

    /// Approve the current phase to allow transition
    ApprovePhase {
        /// Approval notes
        #[arg(long)]
        notes: Option<String>,
    },

    /// Transition to the next phase
    TransitionPhase,

    /// Show execution waves
    Waves,
}
