//! Issue detection and categorization for code review
//!
//! Detects security, performance, style, and other code quality issues.

use super::diff_parser::{DiffHunk, DiffLineType};
use regex_lite::Regex;

/// Severity level of an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Severity {
    Low,       // Nitpicks, suggestions
    Medium,    // Style issues, minor bugs
    High,      // Major bugs, performance issues
    Critical,  // Security vulnerabilities, crashes
}

impl Severity {
    /// Get severity as a number for comparison
    pub fn as_number(&self) -> u8 {
        match self {
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        }
    }

    /// Get severity name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
            Severity::Critical => "CRITICAL",
        }
    }
}

/// Category of issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCategory {
    Security,
    Performance,
    Style,
    Correctness,
    Testing,
    Documentation,
    Complexity,
    Maintainability,
}

impl IssueCategory {
    /// Get category name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueCategory::Security => "Security",
            IssueCategory::Performance => "Performance",
            IssueCategory::Style => "Style",
            IssueCategory::Correctness => "Correctness",
            IssueCategory::Testing => "Testing",
            IssueCategory::Documentation => "Documentation",
            IssueCategory::Complexity => "Complexity",
            IssueCategory::Maintainability => "Maintainability",
        }
    }
}

/// A code quality issue found during review
#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: Severity,
    pub category: IssueCategory,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub message: String,
    pub suggestion: Option<String>,
    pub code_snippet: Option<String>,
}

impl Issue {
    /// Create a new issue
    pub fn new(
        severity: Severity,
        category: IssueCategory,
        file_path: String,
        line_start: usize,
        message: String,
    ) -> Self {
        Issue {
            severity,
            category,
            file_path,
            line_start,
            line_end: line_start,
            message,
            suggestion: None,
            code_snippet: None,
        }
    }

    /// Add a suggestion to this issue
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Add a code snippet to this issue
    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.code_snippet = Some(snippet);
        self
    }
}

/// Detector for code quality issues
pub struct IssueDetector;

impl IssueDetector {
    /// Detect all issues in the given hunks
    pub fn detect_issues(hunks: &[DiffHunk]) -> Vec<Issue> {
        let mut issues = Vec::new();

        for hunk in hunks {
            issues.extend(Self::detect_security(hunk));
            issues.extend(Self::detect_performance(hunk));
            issues.extend(Self::detect_style(hunk));
            issues.extend(Self::detect_correctness(hunk));
            issues.extend(Self::detect_complexity(hunk));
            issues.extend(Self::detect_testing(hunk));
            issues.extend(Self::detect_documentation(hunk));
        }

        // Sort by severity (highest first) and then by line number
        issues.sort_by(|a, b| {
            b.severity
                .as_number()
                .cmp(&a.severity.as_number())
                .then_with(|| a.line_start.cmp(&b.line_start))
        });

        issues
    }

    /// Detect security issues
    fn detect_security(hunk: &DiffHunk) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Patterns for security issues
        let secret_patterns = vec![
            (r#"api[_-]?key\s*=\s*["'][\w-]+["']"#, "Hardcoded API key detected"),
            (r#"password\s*=\s*["'][\w]+["']"#, "Hardcoded password detected"),
            (r#"token\s*=\s*["'][\w-]+["']"#, "Hardcoded token detected"),
            (r#"secret\s*=\s*["'][\w-]+["']"#, "Hardcoded secret detected"),
        ];

        let injection_patterns = vec![
            (r#"format!\(.*\+\s*\w"#, "Potential SQL injection via string concatenation"),
            (r#"execute\s*\(\s*.*\+\s*\w"#, "Potential command injection via string concatenation"),
        ];

        for (line_idx, line) in hunk.lines.iter().enumerate() {
            if line.line_type != DiffLineType::Added {
                continue;
            }

            // Check for hardcoded secrets
            for (pattern, message) in &secret_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(&line.content) {
                        let issue = Issue::new(
                            Severity::Critical,
                            IssueCategory::Security,
                            hunk.file_path.clone(),
                            line.line_number.unwrap_or(0),
                            message.to_string(),
                        )
                        .with_suggestion("Use environment variables or a secret manager.".to_string())
                        .with_snippet(line.content.clone());

                        issues.push(issue);
                    }
                }
            }

            // Check for injection vulnerabilities
            for (pattern, message) in &injection_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(&line.content) {
                        let issue = Issue::new(
                            Severity::Critical,
                            IssueCategory::Security,
                            hunk.file_path.clone(),
                            line.line_number.unwrap_or(0),
                            message.to_string(),
                        )
                        .with_suggestion("Use parameterized queries or proper escaping.".to_string())
                        .with_snippet(line.content.clone());

                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }

    /// Detect performance issues
    fn detect_performance(hunk: &DiffHunk) -> Vec<Issue> {
        let mut issues = Vec::new();

        let performance_patterns = vec![
            (r#"\.clone\(\)"#, "Unnecessary clone detected"),
            (r#"for\s+\w+\s+in\s+.*\.iter\(\)\s*\{[^}]*\.get\("#, "Inefficient lookup inside loop"),
            (r#"for\s+\w+\s+in\s+0\.\.\.", "Consider using iterators instead of range loops"),
        ];

        for line in &hunk.lines {
            if line.line_type != DiffLineType::Added {
                continue;
            }

            for (pattern, message) in &performance_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(&line.content) {
                        let issue = Issue::new(
                            Severity::High,
                            IssueCategory::Performance,
                            hunk.file_path.clone(),
                            line.line_number.unwrap_or(0),
                            message.to_string(),
                        )
                        .with_suggestion("Review algorithm for optimization opportunities.".to_string())
                        .with_snippet(line.content.clone());

                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }

    /// Detect style issues
    fn detect_style(hunk: &DiffHunk) -> Vec<Issue> {
        let mut issues = Vec::new();

        for line in &hunk.lines {
            if line.line_type != DiffLineType::Added {
                continue;
            }

            // Check line length
            if line.content.len() > 100 {
                let issue = Issue::new(
                    Severity::Medium,
                    IssueCategory::Style,
                    hunk.file_path.clone(),
                    line.line_number.unwrap_or(0),
                    format!("Line too long: {} characters", line.content.len()),
                )
                .with_suggestion("Break line into multiple lines (max 100 characters).".to_string())
                .with_snippet(format!("{}...", &line.content[..40.min(line.content.len())]));

                issues.push(issue);
            }

            // Check for TODO/FIXME comments
            if line.content.contains("TODO") || line.content.contains("FIXME") {
                let issue = Issue::new(
                    Severity::Low,
                    IssueCategory::Style,
                    hunk.file_path.clone(),
                    line.line_number.unwrap_or(0),
                    "TODO/FIXME comment detected".to_string(),
                )
                .with_suggestion("Address the TODO or create an issue to track it.".to_string())
                .with_snippet(line.content.clone());

                issues.push(issue);
            }
        }

        issues
    }

    /// Detect correctness issues
    fn detect_correctness(hunk: &DiffHunk) -> Vec<Issue> {
        let mut issues = Vec::new();

        let correctness_patterns = vec![
            (r#"\.unwrap\(\)"#, "Potential panic on unwrap()"),
            (r#"\.expect\(""\)"#, "Empty expect message"),
            (r#"panic!\("#, "Explicit panic"),
            (r#"unsafe\s+"#, "Unsafe code block"),
            (r#"\.parse\(\)\.unwrap\(\)"#, "Unwrapping parse result"),
        ];

        for line in &hunk.lines {
            if line.line_type != DiffLineType::Added {
                continue;
            }

            for (pattern, message) in &correctness_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(&line.content) {
                        let issue = Issue::new(
                            Severity::High,
                            IssueCategory::Correctness,
                            hunk.file_path.clone(),
                            line.line_number.unwrap_or(0),
                            message.to_string(),
                        )
                        .with_suggestion("Use proper error handling with ? operator or match.".to_string())
                        .with_snippet(line.content.clone());

                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }

    /// Detect complexity issues
    fn detect_complexity(hunk: &DiffHunk) -> Vec<Issue> {
        let mut issues = Vec::new();

        let mut nesting_level = 0;
        let mut function_lines = 0;
        let mut function_start = 0;

        for line in &hunk.lines {
            if line.line_type != DiffLineType::Added {
                continue;
            }

            // Track nesting
            let open_braces = line.content.matches('{').count();
            let close_braces = line.content.matches('}').count();

            if open_braces > 0 {
                nesting_level += open_braces;
            }

            // Check for deep nesting
            if nesting_level > 4 {
                let issue = Issue::new(
                    Severity::Medium,
                    IssueCategory::Complexity,
                    hunk.file_path.clone(),
                    line.line_number.unwrap_or(0),
                    format!("Deep nesting (level {})", nesting_level),
                )
                .with_suggestion("Extract nested logic into separate functions.".to_string())
                .with_snippet(line.content.clone());

                issues.push(issue);
            }

            // Update function line count
            if function_lines == 0 {
                function_start = line.line_number.unwrap_or(0);
            }
            function_lines += 1;

            if close_braces > 0 {
                nesting_level = nesting_level.saturating_sub(close_braces);
            }

            // Check for long functions
            if close_braces > 0 && function_lines > 50 {
                let issue = Issue::new(
                    Severity::Medium,
                    IssueCategory::Complexity,
                    hunk.file_path.clone(),
                    function_start,
                    format!("Long function ({} lines)", function_lines),
                )
                .with_suggestion("Break function into smaller functions (< 50 lines).".to_string());

                issues.push(issue);
                function_lines = 0;
            }
        }

        issues
    }

    /// Detect testing issues
    fn detect_testing(hunk: &DiffHunk) -> Vec<Issue> {
        let mut issues = Vec::new();

        let is_test_file = hunk.file_path.contains("test") ||
                          hunk.file_path.contains("spec") ||
                          hunk.file_path.ends_with("_test.rs");

        // If it's not a test file but has function definitions, suggest tests
        if !is_test_file {
            let fn_re = Regex::new(r"pub\s+fn\s+\w+").unwrap();

            for line in &hunk.lines {
                if line.line_type != DiffLineType::Added {
                    continue;
                }

                if fn_re.is_match(&line.content) {
                    let issue = Issue::new(
                        Severity::High,
                        IssueCategory::Testing,
                        hunk.file_path.clone(),
                        line.line_number.unwrap_or(0),
                        "Public function without tests".to_string(),
                    )
                    .with_suggestion("Add unit tests for this function.".to_string())
                    .with_snippet(line.content.clone());

                    issues.push(issue);
                }
            }
        }

        issues
    }

    /// Detect documentation issues
    fn detect_documentation(hunk: &DiffHunk) -> Vec<Issue> {
        let mut issues = Vec::new();

        let pub_item_re = Regex::new(r"pub\s+(fn|struct|enum|trait|mod)\s+\w+").unwrap();

        for (idx, line) in hunk.lines.iter().enumerate() {
            if line.line_type != DiffLineType::Added {
                continue;
            }

            // Check for undocumented public items
            if pub_item_re.is_match(&line.content) {
                // Look back for doc comments (simplified check)
                let has_doc = if idx > 0 {
                    hunk.lines[idx - 1].content.contains("///") ||
                    hunk.lines[idx - 1].content.contains("//!")
                } else {
                    false
                };

                if !has_doc {
                    let issue = Issue::new(
                        Severity::Low,
                        IssueCategory::Documentation,
                        hunk.file_path.clone(),
                        line.line_number.unwrap_or(0),
                        "Undocumented public item".to_string(),
                    )
                    .with_suggestion("Add documentation comments with ///".to_string())
                    .with_snippet(line.content.clone());

                    issues.push(issue);
                }
            }
        }

        issues
    }

    /// Filter issues by minimum severity
    pub fn filter_by_severity(issues: &[Issue], min_severity: Severity) -> Vec<Issue> {
        issues
            .iter()
            .filter(|issue| issue.severity.as_number() >= min_severity.as_number())
            .cloned()
            .collect()
    }

    /// Count issues by severity
    pub fn count_by_severity(issues: &[Issue]) -> (usize, usize, usize, usize) {
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        for issue in issues {
            match issue.severity {
                Severity::Critical => critical += 1,
                Severity::High => high += 1,
                Severity::Medium => medium += 1,
                Severity::Low => low += 1,
            }
        }

        (critical, high, medium, low)
    }

    /// Count issues by category
    pub fn count_by_category(issues: &[Issue]) -> std::collections::HashMap<IssueCategory, usize> {
        let mut counts = std::collections::HashMap::new();

        for issue in issues {
            *counts.entry(issue.category).or_insert(0) += 1;
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_security_hardcoded_secret() {
        let hunk = DiffHunk {
            file_path: "test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "let api_key = \"sk-1234567890\";".to_string(),
                    line_number: Some(1),
                },
            ],
        };

        let issues = IssueDetector::detect_security(&hunk);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].severity, Severity::Critical);
        assert_eq!(issues[0].category, IssueCategory::Security);
    }

    #[test]
    fn test_detect_correctness_unwrap() {
        let hunk = DiffHunk {
            file_path: "test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "let value = result.unwrap();".to_string(),
                    line_number: Some(1),
                },
            ],
        };

        let issues = IssueDetector::detect_correctness(&hunk);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_style_long_line() {
        let hunk = DiffHunk {
            file_path: "test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "a".repeat(150),
                    line_number: Some(1),
                },
            ],
        };

        let issues = IssueDetector::detect_style(&hunk);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].severity, Severity::Medium);
    }

    #[test]
    fn test_filter_by_severity() {
        let issues = vec![
            Issue::new(Severity::Critical, IssueCategory::Security, "test.rs".to_string(), 1, "Test".to_string()),
            Issue::new(Severity::High, IssueCategory::Correctness, "test.rs".to_string(), 2, "Test".to_string()),
            Issue::new(Severity::Medium, IssueCategory::Style, "test.rs".to_string(), 3, "Test".to_string()),
            Issue::new(Severity::Low, IssueCategory::Documentation, "test.rs".to_string(), 4, "Test".to_string()),
        ];

        let high_and_above = IssueDetector::filter_by_severity(&issues, Severity::High);
        assert_eq!(high_and_above.len(), 2);
    }

    #[test]
    fn test_count_by_severity() {
        let issues = vec![
            Issue::new(Severity::Critical, IssueCategory::Security, "test.rs".to_string(), 1, "Test".to_string()),
            Issue::new(Severity::High, IssueCategory::Correctness, "test.rs".to_string(), 2, "Test".to_string()),
            Issue::new(Severity::Medium, IssueCategory::Style, "test.rs".to_string(), 3, "Test".to_string()),
            Issue::new(Severity::Low, IssueCategory::Documentation, "test.rs".to_string(), 4, "Test".to_string()),
        ];

        let (critical, high, medium, low) = IssueDetector::count_by_severity(&issues);
        assert_eq!(critical, 1);
        assert_eq!(high, 1);
        assert_eq!(medium, 1);
        assert_eq!(low, 1);
    }
}
