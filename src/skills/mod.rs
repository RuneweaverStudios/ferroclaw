//! Skill system — categorized, loadable tool packs.
//!
//! Skills extend Ferroclaw's 7 built-in tools with 70+ optional tools organized
//! into 16 categories. Each skill is a TOML manifest that maps to a `ToolHandler`.
//!
//! Skill types:
//! - `bash`: Delegates to a shell command template with argument interpolation.
//! - `native`: Rust-implemented handler (built-in tools use this path).
//! - `mcp_wrapper`: Forwards to a configured MCP server tool.
//!
//! Compatible with AgentSkills.io manifest format for interop.

pub mod agentskills;
pub mod bundled;
pub mod executor;
pub mod loader;
pub mod manifest;
