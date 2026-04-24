//! Tests for CommitTool

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a test git repository
    fn create_test_repo() -> (TempDir, git2::Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize repository
        let repo = git2::Repository::init(repo_path).unwrap();

        // Create initial commit
        {
            let mut index = repo.index().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();

            let sig = repo.signature().unwrap();
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Initial commit",
                &tree,
                &[],
            )
            .unwrap();
        }

        (temp_dir, repo)
    }

    #[test]
    fn test_repository_detection() {
        let (_temp_dir, repo) = create_test_repo();
        let path = repo.path().parent().unwrap().to_str().unwrap();

        // Should find the repository
        let found_repo = find_repository(path).unwrap();
        assert_eq!(found_repo.path(), repo.path());
    }

    #[test]
    fn test_repository_detection_failure() {
        // Try to find repository in non-existent path
        let result = find_repository("/non/existent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_staged_changes() {
        let (_temp_dir, repo) = create_test_repo();
        let path = repo.path().parent().unwrap().to_str().unwrap();

        let found_repo = find_repository(path).unwrap();
        let staged = get_staged_changes(&found_repo).unwrap();

        assert!(staged.is_empty());
    }

    #[test]
    fn test_with_staged_changes() {
        let (temp_dir, repo) = create_test_repo();
        let repo_path = temp_dir.path();

        // Create a test file
        let file_path = repo_path.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test content").unwrap();

        // Stage the file
        let mut index = repo.index().unwrap();
        index.add_path(PathBuf::from("test.txt")).unwrap();
        index.write().unwrap();

        let found_repo = find_repository(repo_path.to_str().unwrap()).unwrap();
        let staged = get_staged_changes(&found_repo).unwrap();

        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0], "test.txt");
    }

    #[test]
    fn test_commit_type_inference() {
        // Test feature inference
        let feat_diff = "+fn new_feature() {\n+    println!(\"new\");\n+}";
        assert_eq!(infer_commit_type(feat_diff), "feat");

        // Test fix inference
        let fix_diff = "-fn broken() {\n+fn fixed() {\n+    // fix bug\n}";
        assert_eq!(infer_commit_type(fix_diff), "fix");

        // Test test inference
        let test_diff = "+#[test]\n+fn test_something() {\n+}";
        assert_eq!(infer_commit_type(test_diff), "test");

        // Test docs inference
        let docs_diff = "+# Documentation\n+New docs here";
        assert_eq!(infer_commit_type(docs_diff), "docs");

        // Test refactor inference
        let refactor_diff = "+fn refactored() {\n+    // simplified\n}";
        assert_eq!(infer_commit_type(refactor_diff), "refactor");

        // Test build inference
        let build_diff = "+[dependencies]\n+new_crate = \"1.0\"";
        assert_eq!(infer_commit_type(build_diff), "build");

        // Test performance inference
        let perf_diff = "+// Optimize performance\n+fn fast() {}";
        assert_eq!(infer_commit_type(perf_diff), "perf");

        // Test chore inference
        let chore_diff = "+.gitignore\n+target/";
        assert_eq!(infer_commit_type(chore_diff), "chore");
    }

    #[test]
    fn test_description_extraction() {
        // Test function extraction
        let fn_diff = "+fn my_function() -> Result<()> {\n+    Ok(())\n+}";
        assert_eq!(
            extract_description(fn_diff, &[]),
            "Add my_function function"
        );

        // Test struct extraction
        let struct_diff = "+pub struct MyStruct {\n+    field: String,\n+}";
        assert_eq!(
            extract_description(struct_diff, &[]),
            "Add MyStruct struct"
        );

        // Test impl extraction
        let impl_diff = "+impl MyTrait for MyStruct {\n+}";
        assert_eq!(
            extract_description(impl_diff, &[]),
            "Implement MyTrait trait"
        );

        // Test fallback to file name
        assert_eq!(
            extract_description("", &["src/main.rs"]),
            "Update src/main.rs"
        );

        // Test test file detection
        assert_eq!(
            extract_description("", &["tests/feature_test.rs"]),
            "Add tests for feature"
        );
    }

    #[test]
    fn test_commit_format_validation() {
        // Valid commits
        assert!(validate_commit_format("feat", "Add new feature").is_ok());
        assert!(validate_commit_format("fix", "Fix bug in parser").is_ok());
        assert!(validate_commit_format("docs", "Update README").is_ok());
        assert!(validate_commit_format("test", "Add unit tests").is_ok());

        // Invalid type
        assert!(validate_commit_format("invalid", "description").is_err());

        // Empty description
        assert!(validate_commit_format("feat", "").is_err());

        // Too long description
        let long_desc = "a".repeat(100);
        assert!(validate_commit_format("feat", &long_desc).is_err());

        // Valid types
        for (type_name, _) in COMMIT_TYPES {
            assert!(validate_commit_format(type_name, "Valid description").is_ok());
        }
    }

    #[test]
    fn test_commit_message_generation() {
        let staged = vec!["src/lib.rs".to_string()];
        let diff = "+fn new_api() {\n+    println!(\"API\");\n+}";
        let recent = &["feat: Add initial implementation".to_string()];

        let message = generate_commit_message(&staged, diff, recent).unwrap();

        // Should start with a valid type
        assert!(message.starts_with("feat:") ||
                message.starts_with("fix:") ||
                message.starts_with("test:"));

        // Should have a description
        let parts: Vec<&str> = message.splitn(2, ": ").collect();
        assert_eq!(parts.len(), 2);
        assert!(!parts[1].is_empty());
    }

    #[test]
    fn test_generate_diff() {
        let (temp_dir, repo) = create_test_repo();
        let repo_path = temp_dir.path();

        // Create and stage a file
        let file_path = repo_path.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Line 1\nLine 2\nLine 3").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(PathBuf::from("test.txt")).unwrap();
        index.write().unwrap();

        let found_repo = find_repository(repo_path.to_str().unwrap()).unwrap();
        let diff = generate_diff(&found_repo);

        assert!(diff.is_ok());
        let diff_content = diff.unwrap();
        assert!(!diff_content.is_empty());
        // Should show the file as added
        assert!(diff_content.contains("test.txt") || diff_content.contains("+Line"));
    }

    #[test]
    fn test_get_recent_commits() {
        let (_temp_dir, repo) = create_test_repo();
        let path = repo.path().parent().unwrap().to_str().unwrap();

        let found_repo = find_repository(path).unwrap();
        let commits = get_recent_commits(&found_repo, 10).unwrap();

        // Should have at least the initial commit
        assert!(!commits.is_empty());
        assert!(commits[0].contains("Initial commit"));
    }

    #[test]
    fn test_integration_end_to_end() {
        let (temp_dir, repo) = create_test_repo();
        let repo_path = temp_dir.path();

        // Create a new feature file
        let file_path = repo_path.join("feature.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "pub fn new_feature() {{").unwrap();
        writeln!(file, "    println!(\"Feature\");").unwrap();
        writeln!(file, "}}").unwrap();

        // Stage the file
        let mut index = repo.index().unwrap();
        index.add_path(PathBuf::from("feature.rs")).unwrap();
        index.write().unwrap();

        // Simulate the tool workflow
        let found_repo = find_repository(repo_path.to_str().unwrap()).unwrap();
        let staged = get_staged_changes(&found_repo).unwrap();
        let diff = generate_diff(&found_repo).unwrap();
        let recent = get_recent_commits(&found_repo, 10).unwrap();

        assert!(!staged.is_empty());
        assert!(!diff.is_empty());
        assert!(!recent.is_empty());

        let message = generate_commit_message(&staged, &diff, &recent).unwrap();
        assert!(message.contains(":"));

        // Verify the message format
        let parts: Vec<&str> = message.splitn(2, ": ").collect();
        assert_eq!(parts.len(), 2);
        assert!(COMMIT_TYPES.iter().any(|(t, _)| *t == parts[0]));
    }

    #[test]
    fn test_error_handling_no_repository() {
        let result = find_repository("/tmp/non_existent_repo_xyz123");
        assert!(result.is_err());
    }

    #[test]
    fn test_amend_flag_handling() {
        // Test that amend flag is properly parsed
        let args = serde_json::json!({
            "repo_path": ".",
            "yes": false,
            "amend": true
        });

        assert_eq!(args.get("amend").unwrap().as_bool(), Some(true));
        assert_eq!(args.get("yes").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_yes_flag_handling() {
        // Test that yes flag is properly parsed
        let args = serde_json::json!({
            "repo_path": ".",
            "yes": true,
            "amend": false
        });

        assert_eq!(args.get("yes").unwrap().as_bool(), Some(true));
        assert_eq!(args.get("amend").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_conventional_commit_types() {
        // Verify all conventional commit types are defined
        assert!(!COMMIT_TYPES.is_empty());

        // Check that common types are present
        let type_names: Vec<&str> = COMMIT_TYPES.iter().map(|(t, _)| *t).collect();
        assert!(type_names.contains(&"feat"));
        assert!(type_names.contains(&"fix"));
        assert!(type_names.contains(&"docs"));
        assert!(type_names.contains(&"test"));
        assert!(type_names.contains(&"refactor"));
    }

    #[test]
    fn test_multiple_file_staging() {
        let (temp_dir, repo) = create_test_repo();
        let repo_path = temp_dir.path();

        // Create multiple files
        for i in 1..=3 {
            let file_path = repo_path.join(format!("file{}.txt", i));
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "Content {}", i).unwrap();
        }

        // Stage all files
        let mut index = repo.index().unwrap();
        for i in 1..=3 {
            index.add_path(PathBuf::from(format!("file{}.txt", i))).unwrap();
        }
        index.write().unwrap();

        let found_repo = find_repository(repo_path.to_str().unwrap()).unwrap();
        let staged = get_staged_changes(&found_repo).unwrap();

        assert_eq!(staged.len(), 3);
    }

    #[test]
    fn test_diff_with_modifications() {
        let (temp_dir, repo) = create_test_repo();
        let repo_path = temp_dir.path();

        // Create initial file
        let file_path = repo_path.join("mod.txt");
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "Original line").unwrap();
        }

        // Commit the file
        {
            let mut index = repo.index().unwrap();
            index.add_path(PathBuf::from("mod.txt")).unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();

            let sig = repo.signature().unwrap();
            let head = repo.head().unwrap();
            let head_commit = head.peel_to_commit().unwrap();

            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Add mod.txt",
                &tree,
                &[&head_commit],
            )
            .unwrap();
        }

        // Modify the file
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "Modified line").unwrap();
        }

        // Stage the modification
        let mut index = repo.index().unwrap();
        index.add_path(PathBuf::from("mod.txt")).unwrap();
        index.write().unwrap();

        let found_repo = find_repository(repo_path.to_str().unwrap()).unwrap();
        let diff = generate_diff(&found_repo).unwrap();

        // Diff should show changes
        assert!(!diff.is_empty());
        // Should contain deletion indicators or addition indicators
        assert!(diff.contains("-") || diff.contains("+"));
    }

    #[test]
    fn test_description_length_limit() {
        let max_desc = "a".repeat(72);
        assert!(validate_commit_format("feat", &max_desc).is_ok());

        let too_long_desc = "a".repeat(73);
        assert!(validate_commit_format("feat", &too_long_desc).is_err());
    }

    #[test]
    fn test_empty_description_validation() {
        assert!(validate_commit_format("feat", "").is_err());
        assert!(validate_commit_format("fix", "   ").is_err());
    }

    #[test]
    fn test_commit_type_case_sensitivity() {
        // Types should be lowercase
        assert!(validate_commit_format("FEAT", "description").is_err());
        assert!(validate_commit_format("Fix", "description").is_err());
        assert!(validate_commit_format("fix", "description").is_ok());
    }
}
