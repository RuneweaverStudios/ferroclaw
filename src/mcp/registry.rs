//! Unified tool registry that merges MCP-discovered tools with built-in tools.

use crate::mcp::client::McpClient;
use crate::mcp::diet::{SkillSummary, generate_skill_summary, render_all_summaries};
use crate::tool::ToolRegistry;
use crate::types::ToolDefinition;

/// Discover tools from all MCP servers and register them in the tool registry.
pub async fn populate_registry_from_mcp(
    registry: &mut ToolRegistry,
    mcp_client: &McpClient,
) -> Vec<SkillSummary> {
    let all_tools = mcp_client.discover_all_tools(false).await;
    let mut summaries = Vec::new();

    for (server_name, tools) in &all_tools {
        let summary = generate_skill_summary(server_name, tools);
        summaries.push(summary);

        for tool in tools {
            registry.register_mcp_tool(tool.clone(), server_name.clone());
        }
    }

    summaries
}

/// Generate the diet context block for the system prompt.
pub fn build_diet_context(summaries: &[SkillSummary], builtin_tools: &[ToolDefinition]) -> String {
    let mut context = String::from("# Available Tools\n\n");

    // Built-in tools section
    if !builtin_tools.is_empty() {
        context.push_str("## Built-in Tools\n");
        for tool in builtin_tools {
            context.push_str(&format!(
                "- {} -- {}\n",
                tool.compact_signature(),
                truncate_desc(&tool.description, 80)
            ));
        }
        context.push('\n');
    }

    // MCP tools (diet summaries)
    if !summaries.is_empty() {
        context.push_str(&render_all_summaries(summaries));
    }

    context
}

fn truncate_desc(text: &str, max: usize) -> &str {
    if text.len() <= max {
        text
    } else {
        &text[..max]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_diet_context() {
        let tools = vec![ToolDefinition {
            name: "read_file".into(),
            description: "Read file contents".into(),
            input_schema: json!({"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]}),
            server_name: None,
        }];
        let summaries = vec![generate_skill_summary("fs", &tools)];

        let context = build_diet_context(&summaries, &tools);
        assert!(context.contains("Available Tools"));
        assert!(context.contains("read_file"));
    }
}
