//! Code review functionality with diff analysis and quality scoring
//!
//! Provides comprehensive code review capabilities including:
//! - Diff parsing and analysis
//! - Quality scoring (complexity, readability, testing, documentation)
//! - Issue detection and categorization
//! - Review report generation

mod command;
pub use command::*;

pub mod diff_parser;
pub mod quality_analyzer;
pub mod issue_detector;
pub mod reporter;

pub use diff_parser::{DiffParser, DiffHunk, DiffLine, DiffLineType};
pub use quality_analyzer::{QualityAnalyzer, QualityScore};
pub use issue_detector::{IssueDetector, Issue, Severity, IssueCategory};
pub use reporter::{ReviewReport, ReviewSummary, TextReportGenerator, JsonReportGenerator};
