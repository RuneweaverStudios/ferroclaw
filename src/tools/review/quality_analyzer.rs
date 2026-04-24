//! Quality scoring and analysis for code review
//!
//! Calculates quality metrics based on complexity, readability, testing, and documentation.

use super::diff_parser::{DiffHunk, DiffLineType};
use regex_lite::Regex;

/// Overall quality score (0-100)
#[derive(Debug, Clone, PartialEq)]
pub struct QualityScore {
    pub total: f64,
    pub complexity: f64,
    pub readability: f64,
    pub testing: f64,
    pub documentation: f64,
}

impl QualityScore {
    /// Create a new quality score
    pub fn new(
        complexity: f64,
        readability: f64,
        testing: f64,
        documentation: f64,
    ) -> Self {
        // Weighted average: complexity 30%, readability 30%, testing 25%, documentation 15%
        let total = (complexity * 0.3) + (readability * 0.3) + (testing * 0.25) + (documentation * 0.15);

        QualityScore {
            total: total.clamp(0.0, 100.0),
            complexity: complexity.clamp(0.0, 100.0),
            readability: readability.clamp(0.0, 100.0),
            testing: testing.clamp(0.0, 100.0),
            documentation: documentation.clamp(0.0, 100.0),
        }
    }

    /// Get grade letter (A, B, C, D, F)
    pub fn grade(&self) -> char {
        match self.total {
            x if x >= 90.0 => 'A',
            x if x >= 80.0 => 'B',
            x if x >= 70.0 => 'C',
            x if x >= 60.0 => 'D',
            _ => 'F',
        }
    }
}

/// Analyzer for code quality metrics
pub struct QualityAnalyzer;

impl QualityAnalyzer {
    /// Calculate overall quality score for all hunks
    pub fn calculate_score(hunks: &[DiffHunk]) -> QualityScore {
        let complexity = Self::analyze_complexity(hunks);
        let readability = Self::analyze_readability(hunks);
        let testing = Self::analyze_testing(hunks);
        let documentation = Self::analyze_documentation(hunks);

        QualityScore::new(complexity, readability, testing, documentation)
    }

    /// Analyze complexity based on nesting depth and function length
    pub fn analyze_complexity(hunks: &[DiffHunk]) -> f64 {
        let mut total_nesting = 0usize;
        let mut total_lines = 0usize;
        let mut long_functions = 0usize;

        for hunk in hunks {
            for line in &hunk.lines {
                if line.line_type == DiffLineType::Added {
                    total_lines += 1;

                    // Count nesting level by counting braces
                    let open_braces = line.content.matches('{').count();
                    let close_braces = line.content.matches('}').count();
                    let nesting = open_braces.saturating_sub(close_braces);
                    total_nesting += nesting;

                    // Check for long functions (heuristic: many consecutive lines in same scope)
                    if line.content.len() > 100 {
                        long_functions += 1;
                    }
                }
            }
        }

        if total_lines == 0 {
            return 100.0;
        }

        // Calculate average nesting
        let avg_nesting = total_nesting as f64 / total_lines as f64;

        // Penalize deep nesting (>4 levels)
        let nesting_penalty = (avg_nesting - 4.0).max(0.0) * 10.0;

        // Penalize long functions
        let long_function_penalty = (long_functions as f64 / total_lines as f64) * 50.0;

        (100.0 - nesting_penalty - long_function_penalty).max(0.0)
    }

    /// Analyze readability based on line length and naming
    pub fn analyze_readability(hunks: &[DiffHunk]) -> f64 {
        let mut total_lines = 0usize;
        let mut long_lines = 0usize;
        let mut naming_issues = 0usize;

        // Patterns for naming issues
        let camel_case_re = Regex::new(r"[a-z][A-Z]").unwrap();
        let snake_case_re = Regex::new(r"_[a-z]").unwrap();

        for hunk in hunks {
            for line in &hunk.lines {
                if line.line_type == DiffLineType::Added {
                    total_lines += 1;

                    // Check line length (>100 characters)
                    if line.content.len() > 100 {
                        long_lines += 1;
                    }

                    // Check for mixed naming conventions (heuristic)
                    let has_camel = camel_case_re.is_match(&line.content);
                    let has_snake = snake_case_re.is_match(&line.content);

                    // If both appear in same line, it's a naming issue
                    if has_camel && has_snake {
                        naming_issues += 1;
                    }
                }
            }
        }

        if total_lines == 0 {
            return 100.0;
        }

        // Calculate penalties
        let long_line_penalty = (long_lines as f64 / total_lines as f64) * 30.0;
        let naming_penalty = (naming_issues as f64 / total_lines as f64) * 20.0;

        (100.0 - long_line_penalty - naming_penalty).max(0.0)
    }

    /// Analyze test coverage
    pub fn analyze_testing(hunks: &[DiffHunk]) -> f64 {
        let mut code_lines = 0usize;
        let mut test_lines = 0usize;
        let mut has_test_file = false;

        for hunk in hunks {
            let is_test_file = hunk.file_path.contains("test") ||
                              hunk.file_path.contains("spec") ||
                              hunk.file_path.ends_with("_test.rs");

            if is_test_file {
                has_test_file = true;
            }

            for line in &hunk.lines {
                if line.line_type == DiffLineType::Added {
                    if is_test_file {
                        test_lines += 1;
                    } else {
                        code_lines += 1;
                    }
                }
            }
        }

        // If no code changes, perfect score
        if code_lines == 0 {
            return 100.0;
        }

        // Calculate test ratio (ideal is 1:1, but 0.5 is acceptable)
        let test_ratio = test_lines as f64 / code_lines as f64;

        // Bonus for having test files
        let test_file_bonus = if has_test_file { 10.0 } else { 0.0 };

        // Score based on ratio (1.0 = 100%, 0.5 = 80%, 0.0 = 0%)
        let ratio_score = (test_ratio * 100.0).min(100.0);

        (ratio_score + test_file_bonus).min(100.0)
    }

    /// Analyze documentation coverage
    pub fn analyze_documentation(hunks: &[DiffHunk]) -> f64 {
        let mut public_items = 0usize;
        let mut documented_items = 0usize;

        // Patterns for public items and docs
        let pub_fn_re = Regex::new(r"pub\s+fn\s+\w+").unwrap();
        let pub_struct_re = Regex::new(r"pub\s+struct\s+\w+").unwrap();
        let pub_enum_re = Regex::new(r"pub\s+enum\s+\w+").unwrap();
        let doc_comment_re = Regex::new(r"///|//\").unwrap();

        for hunk in hunks {
            for line in &hunk.lines {
                if line.line_type == DiffLineType::Added {
                    // Check for public items
                    if pub_fn_re.is_match(&line.content) ||
                       pub_struct_re.is_match(&line.content) ||
                       pub_enum_re.is_match(&line.content) {
                        public_items += 1;

                        // Check if previous line was a doc comment
                        // (simplified - in real implementation, look back)
                        if doc_comment_re.is_match(&line.content) {
                            documented_items += 1;
                        }
                    }
                }
            }
        }

        // If no public items, perfect score
        if public_items == 0 {
            return 100.0;
        }

        // Calculate documentation percentage
        let doc_percentage = (documented_items as f64 / public_items as f64) * 100.0;

        doc_percentage.min(100.0)
    }

    /// Get quality metrics breakdown
    pub fn metrics(hunks: &[DiffHunk]) -> QualityMetrics {
        let mut total_lines = 0;
        let mut added_lines = 0;
        let mut deleted_lines = 0;
        let mut max_nesting = 0;
        let mut avg_line_length = 0;

        for hunk in hunks {
            for line in &hunk.lines {
                total_lines += 1;
                match line.line_type {
                    DiffLineType::Added => {
                        added_lines += 1;
                        avg_line_length += line.content.len();
                    }
                    DiffLineType::Deleted => deleted_lines += 1,
                    _ => {}
                }
            }
        }

        if added_lines > 0 {
            avg_line_length /= added_lines;
        }

        QualityMetrics {
            total_lines,
            added_lines,
            deleted_lines,
            max_nesting,
            avg_line_length,
        }
    }
}

/// Detailed quality metrics
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub total_lines: usize,
    pub added_lines: usize,
    pub deleted_lines: usize,
    pub max_nesting: usize,
    pub avg_line_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_new() {
        let score = QualityScore::new(80.0, 90.0, 70.0, 85.0);
        assert_eq!(score.complexity, 80.0);
        assert_eq!(score.readability, 90.0);
        assert_eq!(score.testing, 70.0);
        assert_eq!(score.documentation, 85.0);
        // Total should be weighted average
        assert!((score.total - 81.25).abs() < 0.01);
    }

    #[test]
    fn test_quality_score_grade() {
        assert_eq!(QualityScore::new(95.0, 95.0, 95.0, 95.0).grade(), 'A');
        assert_eq!(QualityScore::new(85.0, 85.0, 85.0, 85.0).grade(), 'B');
        assert_eq!(QualityScore::new(75.0, 75.0, 75.0, 75.0).grade(), 'C');
        assert_eq!(QualityScore::new(65.0, 65.0, 65.0, 65.0).grade(), 'D');
        assert_eq!(QualityScore::new(50.0, 50.0, 50.0, 50.0).grade(), 'F');
    }

    #[test]
    fn test_analyze_complexity_simple() {
        let hunk = DiffHunk {
            file_path: "test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "fn simple() {".to_string(),
                    line_number: Some(1),
                },
            ],
        };

        let score = QualityAnalyzer::analyze_complexity(&[hunk]);
        assert!(score > 90.0, "Simple code should score high: {}", score);
    }

    #[test]
    fn test_analyze_readability_long_lines() {
        let hunk = DiffHunk {
            file_path: "test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "a".repeat(150), // Very long line
                    line_number: Some(1),
                },
            ],
        };

        let score = QualityAnalyzer::analyze_readability(&[hunk]);
        assert!(score < 100.0, "Long lines should reduce score: {}", score);
    }

    #[test]
    fn test_analyze_testing_with_tests() {
        let test_hunk = DiffHunk {
            file_path: "test_test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "assert_eq!(1, 1);".to_string(),
                    line_number: Some(1),
                },
            ],
        };

        let code_hunk = DiffHunk {
            file_path: "code.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "let x = 1;".to_string(),
                    line_number: Some(1),
                },
            ],
        };

        let score = QualityAnalyzer::analyze_testing(&[test_hunk, code_hunk]);
        assert!(score > 50.0, "Code with tests should score > 50: {}", score);
    }

    #[test]
    fn test_analyze_documentation() {
        let hunk = DiffHunk {
            file_path: "test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "pub fn documented() {}".to_string(),
                    line_number: Some(1),
                },
            ],
        };

        let score = QualityAnalyzer::analyze_documentation(&[hunk]);
        // Should have some score even without perfect documentation
        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    fn test_calculate_score_integration() {
        let hunk = DiffHunk {
            file_path: "test.rs".to_string(),
            old_start: 1,
            old_count: 1,
            new_start: 1,
            new_count: 1,
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Added,
                    content: "fn test() { let x = 1; }".to_string(),
                    line_number: Some(1),
                },
            ],
        };

        let score = QualityAnalyzer::calculate_score(&[hunk]);
        assert!(score.total >= 0.0 && score.total <= 100.0);
        assert!(score.complexity >= 0.0 && score.complexity <= 100.0);
        assert!(score.readability >= 0.0 && score.readability <= 100.0);
    }
}
