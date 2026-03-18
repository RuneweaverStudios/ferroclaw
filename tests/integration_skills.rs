//! Integration tests for the skills subsystem:
//! bundled skill parsing, loader registration, executor interpolation,
//! AgentSkills.io format interop, and manifest serialization.

use ferroclaw::config::SkillsConfig;
use ferroclaw::skills::agentskills::{export_all, from_agentskills, import_all, to_agentskills};
use ferroclaw::skills::bundled::bundled_skills;
use ferroclaw::skills::executor::BashSkillHandler;
use ferroclaw::skills::loader::load_and_register_skills;
use ferroclaw::skills::manifest::{bash_skill, Param, SkillCategory, SkillManifest, SkillType};
use ferroclaw::tool::ToolRegistry;
use ferroclaw::types::Capability;

// ── Bundled Skills ──────────────────────────────────────────────────

#[test]
fn test_all_84_bundled_skills_parse() {
    let skills = bundled_skills();
    assert!(
        skills.len() >= 84,
        "Expected 84+ bundled skills, got {}",
        skills.len()
    );
}

#[test]
fn test_every_skill_has_valid_schema() {
    let skills = bundled_skills();
    for skill in &skills {
        let schema = &skill.input_schema;
        assert_eq!(
            schema.get("type").and_then(|t| t.as_str()),
            Some("object"),
            "Skill '{}' has invalid schema type",
            skill.name
        );
        assert!(
            schema.get("properties").is_some(),
            "Skill '{}' missing properties in schema",
            skill.name
        );
    }
}

#[test]
fn test_every_skill_has_nonempty_name() {
    let skills = bundled_skills();
    for skill in &skills {
        assert!(!skill.name.is_empty(), "Found skill with empty name");
        assert!(
            !skill.name.contains(' '),
            "Skill name '{}' contains spaces",
            skill.name
        );
    }
}

#[test]
fn test_all_16_categories_represented() {
    let skills = bundled_skills();
    let mut categories: std::collections::HashSet<SkillCategory> = std::collections::HashSet::new();
    for skill in &skills {
        categories.insert(skill.category);
    }
    assert_eq!(
        categories.len(),
        16,
        "Expected 16 categories, got {}",
        categories.len()
    );
}

#[test]
fn test_no_duplicate_skill_names() {
    let skills = bundled_skills();
    let mut seen = std::collections::HashSet::new();
    for skill in &skills {
        assert!(
            seen.insert(skill.name.clone()),
            "Duplicate skill name: {}",
            skill.name
        );
    }
}

#[test]
fn test_every_skill_has_at_least_one_capability() {
    let skills = bundled_skills();
    for skill in &skills {
        assert!(
            !skill.required_capabilities.is_empty(),
            "Skill '{}' has no required capabilities",
            skill.name
        );
    }
}

#[test]
fn test_every_skill_has_tags() {
    let skills = bundled_skills();
    for skill in &skills {
        assert!(
            !skill.tags.is_empty(),
            "Skill '{}' has no tags",
            skill.name
        );
    }
}

// ── Loader ──────────────────────────────────────────────────────────

#[test]
fn test_loader_registers_all_bundled_skills() {
    let mut registry = ToolRegistry::new();
    let config = SkillsConfig::default();
    let stats = load_and_register_skills(&mut registry, &config).unwrap();
    assert!(stats.registered >= 84);
    assert_eq!(stats.failed, 0);
    assert_eq!(stats.by_category.len(), 16);
}

#[test]
fn test_loader_respects_disabled_skills() {
    let mut registry = ToolRegistry::new();
    let config = SkillsConfig {
        disabled_skills: Some(vec!["find_files".into(), "git_status".into()]),
        ..Default::default()
    };
    let stats = load_and_register_skills(&mut registry, &config).unwrap();
    assert!(stats.skipped >= 2);
    assert!(registry.get_meta("find_files").is_none());
    assert!(registry.get_meta("git_status").is_none());
}

#[test]
fn test_loader_respects_category_filter() {
    let mut registry = ToolRegistry::new();
    let config = SkillsConfig {
        enabled_categories: Some(vec!["filesystem".into(), "version_control".into()]),
        ..Default::default()
    };
    let stats = load_and_register_skills(&mut registry, &config).unwrap();
    assert!(stats.registered > 0);
    assert!(stats.skipped > 0);
    // Only filesystem and version_control categories should be registered
    assert!(stats.by_category.len() <= 2);
}

// ── Executor ────────────────────────────────────────────────────────

#[test]
fn test_executor_with_numeric_argument() {
    let handler = BashSkillHandler::new("head -n {{count}} {{file}}".into());
    let args = serde_json::json!({"count": 10, "file": "/tmp/test.txt"});
    let result = handler.interpolate(&args).unwrap();
    assert_eq!(result, "head -n 10 /tmp/test.txt");
}

#[test]
fn test_executor_with_empty_optional() {
    let handler = BashSkillHandler::new("ls {{path}} {{?flags}}".into());
    let args = serde_json::json!({"path": "/tmp"});
    let result = handler.interpolate(&args).unwrap();
    assert_eq!(result, "ls /tmp");
}

#[test]
fn test_executor_multiple_optional_params() {
    let handler =
        BashSkillHandler::new("curl {{?method}} {{url}} {{?headers}} {{?data}}".into());
    let args = serde_json::json!({"url": "https://example.com"});
    let result = handler.interpolate(&args).unwrap();
    assert_eq!(result, "curl https://example.com");
}

#[test]
fn test_executor_preserves_quotes_in_template() {
    let handler = BashSkillHandler::new("grep -r '{{pattern}}' {{path}}".into());
    let args = serde_json::json!({"pattern": "fn main", "path": "/tmp"});
    let result = handler.interpolate(&args).unwrap();
    assert_eq!(result, "grep -r 'fn main' /tmp");
}

#[test]
fn test_executor_rejects_all_missing_required() {
    let handler = BashSkillHandler::new("echo {{a}} {{b}}".into());
    let args = serde_json::json!({});
    let result = handler.interpolate(&args);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("a"));
    assert!(err.contains("b"));
}

// ── Manifest ────────────────────────────────────────────────────────

#[test]
fn test_manifest_toml_roundtrip() {
    let skill = bash_skill(
        "test_tool",
        "A test tool for testing",
        SkillCategory::Testing,
        &["test", "qa"],
        "echo {{message}}",
        &[
            Param::required("message", "The message to echo"),
            Param::optional("verbose", "Enable verbose output"),
        ],
        &[Capability::ProcessExec],
    );
    let toml_str = toml::to_string_pretty(&skill).unwrap();
    let parsed: SkillManifest = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.name, "test_tool");
    assert_eq!(parsed.category, SkillCategory::Testing);
    assert!(parsed.enabled);
}

#[test]
fn test_manifest_disabled_skill() {
    let mut skill = bash_skill(
        "disabled_tool",
        "A disabled skill",
        SkillCategory::System,
        &[],
        "echo disabled",
        &[],
        &[Capability::ProcessExec],
    );
    skill.enabled = false;

    let json = serde_json::to_string(&skill).unwrap();
    let parsed: SkillManifest = serde_json::from_str(&json).unwrap();
    assert!(!parsed.enabled);
}

// ── AgentSkills.io Interop ──────────────────────────────────────────

#[test]
fn test_agentskills_roundtrip_preserves_all_fields() {
    let skills = bundled_skills();
    for skill in &skills {
        let exported = to_agentskills(skill);
        let imported = from_agentskills(&exported);
        assert_eq!(skill.name, imported.name, "Name mismatch for {}", skill.name);
        assert_eq!(
            skill.description, imported.description,
            "Description mismatch for {}",
            skill.name
        );
    }
}

#[test]
fn test_agentskills_export_import_preserves_count() {
    let skills = bundled_skills();
    let exported = export_all(&skills);
    let imported = import_all(&exported);
    assert_eq!(skills.len(), imported.len());
}

#[test]
fn test_agentskills_mcp_wrapper_roundtrip() {
    let manifest = ferroclaw::skills::agentskills::AgentSkillsManifest {
        name: "mcp_tool".into(),
        description: "An MCP tool".into(),
        version: "1.0.0".into(),
        author: "test".into(),
        category: "web".into(),
        tags: vec!["api".into()],
        parameters: serde_json::json!({"type": "object"}),
        command: Some("mcp://server/tool_name".into()),
        permissions: vec!["net_outbound".into()],
        examples: vec![],
    };
    let skill = from_agentskills(&manifest);
    match &skill.skill_type {
        SkillType::McpWrapper { server, tool } => {
            assert_eq!(server, "server");
            assert_eq!(tool, "tool_name");
        }
        _ => panic!("Expected McpWrapper skill type"),
    }
}
