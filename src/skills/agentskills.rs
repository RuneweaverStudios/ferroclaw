//! AgentSkills.io manifest compatibility.
//!
//! Converts between Ferroclaw's SkillManifest and the AgentSkills.io JSON format
//! for interoperability with the broader agent skills ecosystem.

use crate::skills::manifest::{SkillCategory, SkillManifest, SkillType};
use crate::types::Capability;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// AgentSkills.io manifest format (v1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkillsManifest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub parameters: Value,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub examples: Vec<AgentSkillExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkillExample {
    pub input: Value,
    pub description: String,
}

/// Convert a Ferroclaw SkillManifest to AgentSkills.io format.
pub fn to_agentskills(skill: &SkillManifest) -> AgentSkillsManifest {
    let command = match &skill.skill_type {
        SkillType::Bash { command_template } => Some(command_template.clone()),
        SkillType::Native => None,
        SkillType::McpWrapper { server, tool } => Some(format!("mcp://{server}/{tool}")),
    };

    let permissions = skill
        .required_capabilities
        .iter()
        .map(|c| c.to_string())
        .collect();

    AgentSkillsManifest {
        name: skill.name.clone(),
        description: skill.description.clone(),
        version: skill.version.clone(),
        author: "ferroclaw".into(),
        category: format!("{:?}", skill.category).to_lowercase(),
        tags: skill.tags.clone(),
        parameters: skill.input_schema.clone(),
        command,
        permissions,
        examples: Vec::new(),
    }
}

/// Convert an AgentSkills.io manifest to Ferroclaw SkillManifest.
pub fn from_agentskills(manifest: &AgentSkillsManifest) -> SkillManifest {
    let category = parse_category(&manifest.category);

    let skill_type = match &manifest.command {
        Some(cmd) if cmd.starts_with("mcp://") => {
            let parts: Vec<&str> = cmd.trim_start_matches("mcp://").splitn(2, '/').collect();
            SkillType::McpWrapper {
                server: parts.first().unwrap_or(&"").to_string(),
                tool: parts.get(1).unwrap_or(&"").to_string(),
            }
        }
        Some(cmd) => SkillType::Bash {
            command_template: cmd.clone(),
        },
        None => SkillType::Native,
    };

    let required_capabilities = manifest
        .permissions
        .iter()
        .filter_map(|p| parse_capability(p))
        .collect();

    SkillManifest {
        name: manifest.name.clone(),
        description: manifest.description.clone(),
        version: if manifest.version.is_empty() {
            "0.1.0".into()
        } else {
            manifest.version.clone()
        },
        category,
        tags: manifest.tags.clone(),
        skill_type,
        input_schema: manifest.parameters.clone(),
        required_capabilities,
        enabled: true,
    }
}

/// Export all bundled skills as AgentSkills.io JSON.
pub fn export_all(skills: &[SkillManifest]) -> Value {
    let manifests: Vec<AgentSkillsManifest> = skills.iter().map(to_agentskills).collect();
    serde_json::to_value(manifests).unwrap_or_default()
}

/// Import skills from AgentSkills.io JSON array.
pub fn import_all(json: &Value) -> Vec<SkillManifest> {
    match serde_json::from_value::<Vec<AgentSkillsManifest>>(json.clone()) {
        Ok(manifests) => manifests.iter().map(from_agentskills).collect(),
        Err(_) => Vec::new(),
    }
}

fn parse_category(s: &str) -> SkillCategory {
    match s.to_lowercase().as_str() {
        "filesystem" | "file" | "fs" => SkillCategory::Filesystem,
        "version_control" | "vcs" | "git" => SkillCategory::VersionControl,
        "code_analysis" | "code" | "analysis" => SkillCategory::CodeAnalysis,
        "web" | "http" | "api" => SkillCategory::Web,
        "database" | "db" | "sql" => SkillCategory::Database,
        "docker" | "container" | "containers" => SkillCategory::Docker,
        "kubernetes" | "k8s" => SkillCategory::Kubernetes,
        "system" | "os" | "process" => SkillCategory::System,
        "text_processing" | "text" | "parsing" => SkillCategory::TextProcessing,
        "network" | "net" | "networking" => SkillCategory::Network,
        "security" | "sec" | "crypto" => SkillCategory::Security,
        "documentation" | "docs" | "doc" => SkillCategory::Documentation,
        "testing" | "test" | "qa" => SkillCategory::Testing,
        "package_management" | "packages" | "deps" => SkillCategory::PackageManagement,
        "cloud" | "infra" | "infrastructure" => SkillCategory::Cloud,
        "media" | "image" | "audio" | "video" => SkillCategory::Media,
        _ => SkillCategory::System,
    }
}

fn parse_capability(s: &str) -> Option<Capability> {
    match s {
        "fs_read" => Some(Capability::FsRead),
        "fs_write" => Some(Capability::FsWrite),
        "net_outbound" => Some(Capability::NetOutbound),
        "net_listen" => Some(Capability::NetListen),
        "process_exec" => Some(Capability::ProcessExec),
        "memory_read" => Some(Capability::MemoryRead),
        "memory_write" => Some(Capability::MemoryWrite),
        "browser_control" => Some(Capability::BrowserControl),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::bundled::bundled_skills;

    #[test]
    fn test_roundtrip_conversion() {
        let skills = bundled_skills();
        let first = &skills[0];
        let agentskills = to_agentskills(first);
        let back = from_agentskills(&agentskills);
        assert_eq!(first.name, back.name);
        assert_eq!(first.description, back.description);
    }

    #[test]
    fn test_export_import_all() {
        let skills = bundled_skills();
        let exported = export_all(&skills);
        let imported = import_all(&exported);
        assert_eq!(skills.len(), imported.len());
    }

    #[test]
    fn test_category_parsing() {
        assert_eq!(parse_category("filesystem"), SkillCategory::Filesystem);
        assert_eq!(parse_category("git"), SkillCategory::VersionControl);
        assert_eq!(parse_category("k8s"), SkillCategory::Kubernetes);
        assert_eq!(parse_category("unknown"), SkillCategory::System);
    }
}
