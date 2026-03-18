//! Skill manifest types.

use crate::types::Capability;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A skill manifest defines a single tool with metadata, category, and execution strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: SkillCategory,
    pub tags: Vec<String>,
    pub skill_type: SkillType,
    pub input_schema: Value,
    pub required_capabilities: Vec<Capability>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// How a skill executes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SkillType {
    /// Shell command with `{{arg}}` (required) and `{{?arg}}` (optional) interpolation.
    Bash { command_template: String },
    /// Rust-native handler (registered separately).
    Native,
    /// Forwards to a configured MCP server tool.
    McpWrapper { server: String, tool: String },
}

/// Skill categories — 16 domains covering common agent tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Filesystem,
    VersionControl,
    CodeAnalysis,
    Web,
    Database,
    Docker,
    Kubernetes,
    System,
    TextProcessing,
    Network,
    Security,
    Documentation,
    Testing,
    PackageManagement,
    Cloud,
    Media,
}

impl SkillCategory {
    pub fn all() -> &'static [SkillCategory] {
        &[
            Self::Filesystem,
            Self::VersionControl,
            Self::CodeAnalysis,
            Self::Web,
            Self::Database,
            Self::Docker,
            Self::Kubernetes,
            Self::System,
            Self::TextProcessing,
            Self::Network,
            Self::Security,
            Self::Documentation,
            Self::Testing,
            Self::PackageManagement,
            Self::Cloud,
            Self::Media,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Filesystem => "Filesystem",
            Self::VersionControl => "Version Control",
            Self::CodeAnalysis => "Code Analysis",
            Self::Web => "Web & HTTP",
            Self::Database => "Database",
            Self::Docker => "Docker & Containers",
            Self::Kubernetes => "Kubernetes",
            Self::System => "System & OS",
            Self::TextProcessing => "Text Processing",
            Self::Network => "Network",
            Self::Security => "Security",
            Self::Documentation => "Documentation",
            Self::Testing => "Testing",
            Self::PackageManagement => "Package Management",
            Self::Cloud => "Cloud & Infrastructure",
            Self::Media => "Media & Files",
        }
    }
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Helper to build a bash-type skill manifest with minimal boilerplate.
pub fn bash_skill(
    name: &str,
    description: &str,
    category: SkillCategory,
    tags: &[&str],
    command_template: &str,
    params: &[Param],
    capabilities: &[Capability],
) -> SkillManifest {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for p in params {
        let mut prop = serde_json::Map::new();
        prop.insert("type".into(), Value::String(p.param_type.to_string()));
        prop.insert("description".into(), Value::String(p.description.to_string()));
        if let Some(ref default) = p.default {
            prop.insert("default".into(), Value::String(default.clone()));
        }
        properties.insert(p.name.to_string(), Value::Object(prop));
        if p.required {
            required.push(Value::String(p.name.to_string()));
        }
    }

    let input_schema = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required,
    });

    SkillManifest {
        name: name.into(),
        description: description.into(),
        version: "0.1.0".into(),
        category,
        tags: tags.iter().map(|s| s.to_string()).collect(),
        skill_type: SkillType::Bash {
            command_template: command_template.into(),
        },
        input_schema,
        required_capabilities: capabilities.to_vec(),
        enabled: true,
    }
}

/// Parameter definition for skill schema generation.
pub struct Param {
    pub name: &'static str,
    pub description: &'static str,
    pub param_type: &'static str,
    pub required: bool,
    pub default: Option<String>,
}

impl Param {
    pub fn required(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            param_type: "string",
            required: true,
            default: None,
        }
    }

    pub fn optional(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            param_type: "string",
            required: false,
            default: None,
        }
    }

    pub fn with_default(mut self, default: &str) -> Self {
        self.default = Some(default.into());
        self
    }

    pub fn typed(mut self, param_type: &'static str) -> Self {
        self.param_type = param_type;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_category_count() {
        assert_eq!(SkillCategory::all().len(), 16);
    }

    #[test]
    fn test_bash_skill_builder() {
        let skill = bash_skill(
            "test_skill",
            "A test skill",
            SkillCategory::System,
            &["test"],
            "echo {{message}}",
            &[Param::required("message", "Message to echo")],
            &[Capability::ProcessExec],
        );
        assert_eq!(skill.name, "test_skill");
        assert_eq!(skill.category, SkillCategory::System);
        assert!(skill.enabled);

        let required = skill
            .input_schema
            .get("required")
            .and_then(|r| r.as_array())
            .unwrap();
        assert_eq!(required.len(), 1);
    }

    #[test]
    fn test_skill_manifest_serialization() {
        let skill = bash_skill(
            "find_files",
            "Find files",
            SkillCategory::Filesystem,
            &["find"],
            "find {{path}} -name '{{pattern}}'",
            &[
                Param::required("path", "Directory"),
                Param::required("pattern", "Glob pattern"),
            ],
            &[Capability::ProcessExec],
        );
        let json = serde_json::to_string(&skill).unwrap();
        let parsed: SkillManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "find_files");
    }
}
