//! Review report generation and formatting
//!
//! Provides structured report generation in text and JSON formats.

use super::{Issue, Severity, QualityScore, diff_parser::DiffStats};
use serde_json::json;

/// Summary of review findings
#[derive(Debug, Clone)]
pub struct ReviewSummary {
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub issues_count: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

/// Comprehensive review report
#[derive(Debug, Clone)]
pub struct ReviewReport {
    pub summary: ReviewSummary,
    pub issues: Vec<Issue>,
    pub quality_score: QualityScore,
    pub recommendations: Vec<String>,
    pub diff_stats: DiffStats,
}

/// Generate human-readable text reports
pub struct TextReportGenerator;

impl TextReportGenerator {
    /// Generate a text report
    pub fn generate(report: &ReviewReport) -> String {
        let mut output = String::new();

        // Header
        output.push_str("╔════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                      Code Review Report                     ║\n");
        output.push_str("╚════════════════════════════════════════════════════════════╝\n\n");

        // Summary section
        Self::write_summary(&mut output, report);

        // Issues section
        if !report.issues.is_empty() {
            Self::write_issues(&mut output, report);
        } else {
            output.push_str("\n✅ No issues found! Great job!\n\n");
        }

        // Quality score section
        Self::write_quality_score(&mut output, report);

        // Recommendations section
        if !report.recommendations.is_empty() {
            Self::write_recommendations(&mut output, report);
        }

        output
    }

    /// Write summary section
    fn write_summary(output: &mut String, report: &ReviewReport) {
        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str("SUMMARY\n");
        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str(&format!("Files changed:  {}\n", report.summary.files_changed));
        output.push_str(&format!("Lines added:    {}\n", report.summary.lines_added));
        output.push_str(&format!("Lines deleted:  {}\n", report.summary.lines_deleted));
        output.push_str(&format!("Issues found:   {}\n", report.summary.issues_count));
        output.push_str(&format!(
            "  ├─ CRITICAL: {}\n",
            report.summary.critical_count
        ));
        output.push_str(&format!(
            "  ├─ HIGH:     {}\n",
            report.summary.high_count
        ));
        output.push_str(&format!(
            "  ├─ MEDIUM:   {}\n",
            report.summary.medium_count
        ));
        output.push_str(&format!(
            "  └─ LOW:      {}\n",
            report.summary.low_count
        ));
        output.push_str("\n");
    }

    /// Write issues section
    fn write_issues(output: &mut String, report: &ReviewReport) {
        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str("ISSUES\n");
        output.push_str("═══════════════════════════════════════════════════════════\n");

        for (idx, issue) in report.issues.iter().enumerate() {
            let severity_icon = match issue.severity {
                Severity::Critical => "🚨",
                Severity::High => "⚠️",
                Severity::Medium => "⚡",
                Severity::Low => "💡",
            };

            output.push_str(&format!(
                "\n[{}{}] {}:{}\n",
                severity_icon,
                issue.severity.as_str(),
                issue.file_path,
                issue.line_start
            ));
            output.push_str(&format!("  Category: {}\n", issue.category.as_str()));
            output.push_str(&format!("  Message:  {}\n", issue.message));

            if let Some(ref suggestion) = issue.suggestion {
                output.push_str(&format!("  💡 Suggestion: {}\n", suggestion));
            }

            if let Some(ref snippet) = issue.code_snippet {
                let truncated = if snippet.len() > 80 {
                    format!("{}...", &snippet[..77])
                } else {
                    snippet.clone()
                };
                output.push_str(&format!("  📝 Code: {}\n", truncated));
            }

            // Separator between issues
            if idx < report.issues.len() - 1 {
                output.push_str("  ──────────────────────────────────────────────────────");
            }
        }

        output.push_str("\n\n");
    }

    /// Write quality score section
    fn write_quality_score(output: &mut String, report: &ReviewReport) {
        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str("QUALITY SCORE\n");
        output.push_str("═══════════════════════════════════════════════════════════\n");

        let score = &report.quality_score;
        let grade_icon = match score.grade() {
            'A' => "🌟",
            'B' => "👍",
            'C' => "👌",
            'D' => "👀",
            'F' => "🔥",
            _ => "?",
        };

        output.push_str(&format!(
            "Total Score: {:.1}/100 (Grade: {}{})\n\n",
            score.total,
            grade_icon,
            score.grade()
        ));

        // Individual scores with bars
        output.push_str(&format!(
            "  Complexity:    {:.0}/100 {}\n",
            score.complexity,
            Self::score_bar(score.complexity)
        ));
        output.push_str(&format!(
            "  Readability:  {:.0}/100 {}\n",
            score.readability,
            Self::score_bar(score.readability)
        ));
        output.push_str(&format!(
            "  Testing:       {:.0}/100 {}\n",
            score.testing,
            Self::score_bar(score.testing)
        ));
        output.push_str(&format!(
            "  Documentation: {:.0}/100 {}\n",
            score.documentation,
            Self::score_bar(score.documentation)
        ));

        output.push_str("\n");
    }

    /// Write recommendations section
    fn write_recommendations(output: &mut String, report: &ReviewReport) {
        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str("RECOMMENDATIONS\n");
        output.push_str("═══════════════════════════════════════════════════════════\n");

        for (idx, rec) in report.recommendations.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", idx + 1, rec));
        }

        output.push_str("\n");
    }

    /// Generate a visual score bar
    fn score_bar(score: f64) -> String {
        let filled = (score / 10.0) as usize;
        let empty = 10 - filled;
        format!(
            "[{}{}]",
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }
}

/// Generate JSON reports
pub struct JsonReportGenerator;

impl JsonReportGenerator {
    /// Generate a JSON report
    pub fn generate(report: &ReviewReport) -> String {
        let issues_json: Vec<serde_json::Value> = report
            .issues
            .iter()
            .map(|issue| {
                json!({
                    "severity": issue.severity.as_str(),
                    "category": issue.category.as_str(),
                    "file_path": issue.file_path,
                    "line_start": issue.line_start,
                    "line_end": issue.line_end,
                    "message": issue.message,
                    "suggestion": issue.suggestion,
                    "code_snippet": issue.code_snippet,
                })
            })
            .collect();

        let output = json!({
            "summary": {
                "files_changed": report.summary.files_changed,
                "lines_added": report.summary.lines_added,
                "lines_deleted": report.summary.lines_deleted,
                "issues_count": report.summary.issues_count,
                "critical_count": report.summary.critical_count,
                "high_count": report.summary.high_count,
                "medium_count": report.summary.medium_count,
                "low_count": report.summary.low_count,
            },
            "issues": issues_json,
            "quality_score": {
                "total": report.quality_score.total,
                "complexity": report.quality_score.complexity,
                "readability": report.quality_score.readability,
                "testing": report.quality_score.testing,
                "documentation": report.quality_score.documentation,
                "grade": report.quality_score.grade().to_string(),
            },
            "recommendations": report.recommendations,
            "diff_stats": {
                "files_changed": report.diff_stats.files_changed,
                "insertions": report.diff_stats.insertions,
                "deletions": report.diff_stats.deletions,
            },
        });

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::review::{Issue, IssueCategory};

    #[test]
    fn test_text_report_generation() {
        let report = ReviewReport {
            summary: ReviewSummary {
                files_changed: 2,
                lines_added: 50,
                lines_deleted: 10,
                issues_count: 3,
                critical_count: 1,
                high_count: 1,
                medium_count: 1,
                low_count: 0,
            },
            issues: vec![
                Issue::new(
                    Severity::Critical,
                    IssueCategory::Security,
                    "test.rs".to_string(),
                    10,
                    "Test issue".to_string(),
                ),
            ],
            quality_score: QualityScore::new(75.0, 80.0, 70.0, 75.0),
            recommendations: vec!["Fix the issues".to_string()],
            diff_stats: DiffStats {
                files_changed: 2,
                insertions: 50,
                deletions: 10,
            },
        };

        let text = TextReportGenerator::generate(&report);
        assert!(text.contains("Code Review Report"));
        assert!(text.contains("SUMMARY"));
        assert!(text.contains("Files changed:  2"));
        assert!(text.contains("CRITICAL"));
        assert!(text.contains("QUALITY SCORE"));
        assert!(text.contains("RECOMMENDATIONS"));
    }

    #[test]
    fn test_json_report_generation() {
        let report = ReviewReport {
            summary: ReviewSummary {
                files_changed: 1,
                lines_added: 20,
                lines_deleted: 5,
                issues_count: 1,
                critical_count: 0,
                high_count: 1,
                medium_count: 0,
                low_count: 0,
            },
            issues: vec![],
            quality_score: QualityScore::new(85.0, 90.0, 80.0, 85.0),
            recommendations: vec!["Good job!".to_string()],
            diff_stats: DiffStats {
                files_changed: 1,
                insertions: 20,
                deletions: 5,
            },
        };

        let json = JsonReportGenerator::generate(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["summary"]["files_changed"], 1);
        assert_eq!(parsed["quality_score"]["total"], 85.0);
        assert!(parsed["recommendations"].is_array());
    }

    #[test]
    fn test_score_bar() {
        let bar_100 = TextReportGenerator::score_bar(100.0);
        assert_eq!(bar_100, "[██████████]");

        let bar_50 = TextReportGenerator::score_bar(50.0);
        assert_eq!(bar_50, "[█████░░░░░]");

        let bar_0 = TextReportGenerator::score_bar(0.0);
        assert_eq!(bar_0, "[░░░░░░░░░░]");
    }
}
