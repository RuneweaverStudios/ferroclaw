# Git Workflow - Commit and Review Commands

## Overview

Ferroclaw provides two powerful Git workflow commands:

1. **CommitTool**: Interactive commit workflow with conventional commits
2. **ReviewCommand**: Code review with diff analysis, quality scoring, and issue detection

Both commands integrate with git2-rs for Git operations and provide automated analysis of your changes.

## Commit Command

### Overview

CommitTool creates Git commits with:
- Automatic conventional commit message generation
- Staged changes analysis
- Diff preview
- Interactive approval workflow

### Conventional Commit Format

```
<type>: <description>

[optional body]
```

**Supported Types:**
- `feat` - A new feature
- `fix` - A bug fix
- `docs` - Documentation only changes
- `style` - Changes that do not affect the meaning of the code
- `refactor` - A code change that neither fixes a bug nor adds a feature
- `perf` - A code change that improves performance
- `test` - Adding missing tests or correcting existing tests
- `build` - Changes that affect the build system or external dependencies
- `ci` - Changes to CI configuration files and scripts
- `chore` - Other changes that don't modify src or test files
- `revert` - Reverts a previous commit

### Usage

#### JSON API

```json
{
  "name": "commit",
  "arguments": {
    "yes": false,
    "amend": false,
    "repo_path": "."
  }
}
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `yes` | boolean | false | Auto-approve without confirmation |
| `amend` | boolean | false | Amend previous commit instead of creating new one |
| `repo_path` | string | "." | Path to Git repository |

#### CLI

```bash
# Interactive commit (shows preview)
ferroclaw commit

# Auto-approve (skip confirmation)
ferroclaw commit --yes

# Amend previous commit
ferroclaw commit --amend

# Specify repository path
ferroclaw commit --repo-path /path/to/repo
```

### Workflow

1. **Find Repository**: Locates .git directory
2. **Check Staged Changes**: Validates files are staged
3. **Generate Diff**: Creates unified diff of changes
4. **Analyze Changes**: Examines files, lines added/removed
5. **Generate Message**: Creates conventional commit message
6. **Preview**: Shows proposed commit and diff
7. **Approval**: Waits for user confirmation (unless --yes)
8. **Create Commit**: Executes git commit

### Example Output

```
Proposed commit:

feat: add user authentication with OAuth2

Implement OAuth2 authentication flow with support for
Google and GitHub providers. Includes session management
and token refresh logic.

Files changed:
- src/auth/oauth.rs (new)
- src/auth/session.rs (new)
- src/main.rs (modified)

Diff:
diff --git a/src/auth/oauth.rs b/src/auth/oauth.rs
new file mode 100644
index 0000000..1234567
--- /dev/null
+++ b/src/auth/oauth.rs
@@ -0,0 +1,45 @@
+//! OAuth2 authentication implementation
+
+pub struct OAuthClient {
+    // ...
+}
...

Use --yes flag to auto-approve.
```

### Auto-Approval

```bash
ferroclaw commit --yes
```

Skips confirmation, creates commit immediately.

### Amend Commit

```bash
ferroclaw commit --amend
```

Modifies the most recent commit instead of creating a new one.

## Review Command

### Overview

ReviewCommand performs automated code review with:
- Diff analysis at multiple scopes
- Quality scoring (0-100)
- Issue detection by category and severity
- Actionable recommendations

### Scopes

| Scope | Description | Example |
|-------|-------------|---------|
| `Staged` | Review staged changes (index vs HEAD) | Before committing |
| `WorkingTree` | Review working tree (working dir vs index) | After editing, before staging |
| `CommitRange` | Review commit range | `main..HEAD` |
| `All` | Review all changes | Comprehensive review |

### Usage

#### JSON API

```json
{
  "name": "review",
  "arguments": {
    "scope": "staged",
    "min_severity": "high",
    "file_pattern": "**/*.rs",
    "output": "text"
  }
}
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `scope` | string | "staged" | Review scope (staged/working/commit-range/all) |
| `min_severity` | string | "low" | Minimum severity to show (critical/high/medium/low) |
| `file_pattern` | string | null | Glob pattern to filter files |
| `output` | string | "text" | Output format (text/json) |

#### CLI

```bash
# Review staged changes
ferroclaw review

# Review working tree
ferroclaw review --scope working

# Review commit range
ferroclaw review --scope main..HEAD

# Filter by severity
ferroclaw review --severity high

# Filter by file pattern
ferroclaw review --pattern "**/*.rs"

# JSON output
ferroclaw review --output json
```

### Issue Categories

| Category | Description | Example Issues |
|----------|-------------|----------------|
| **Security** | Injection, auth, crypto | Hardcoded secrets, SQL injection |
| **Performance** | Inefficient algorithms | Nested loops O(n²), missing cache |
| **Style** | Naming, formatting | Line length > 100, inconsistent naming |
| **Correctness** | Logic errors, edge cases | Unwrap() without context, empty error handlers |
| **Testing** | Missing tests, coverage | New functions without tests |
| **Documentation** | Missing docs | Missing pub struct docs |
| **Complexity** | High cyclomatic complexity | Nesting depth > 4, function > 50 lines |
| **Maintainability** | Code duplication, coupling | Duplicated code blocks |

### Severity Levels

| Level | Impact | Examples |
|-------|--------|----------|
| **Critical** | Security vulnerabilities, crashes | Hardcoded API keys, SQL injection |
| **High** | Major bugs, performance issues | Unwrap() panics, O(n²) loops |
| **Medium** | Style issues, minor bugs | Inconsistent naming, missing docs |
| **Low** | Nitpicks, suggestions | Magic numbers, formatting |

### Output Formats

#### Text Output (Default)

```
Review Report
============

Summary
-------
Files changed: 5
Lines added: 234
Lines deleted: 87
Issues found: 12

Issues (Severity: HIGH+)
------------------------
[CRITICAL] src/auth.rs:45
  Category: Security
  Hardcoded API key detected
  Suggestion: Use environment variable

  44:     let api_key = "sk-1234567890";
  45: ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

[HIGH] src/user.rs:123
  Category: Correctness
  Potential panic on unwrap()
  Suggestion: Use ? operator or match

  123:     let user = user.unwrap();
       ^^^^^^^^^^^^^^^^^^^^^^^^

[MEDIUM] src/api.rs:67
  Category: Style
  Line length exceeds 100 characters
  Suggestion: Break line for readability

  67:     let response = client.post(url).json(&body).send().await.map_err(|e| Error::RequestFailed(e))?;
       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Quality Score
-------------
Total: 72/100
  Complexity: 68/100
  Readability: 85/100
  Testing: 60/100
  Documentation: 75/100

Recommendations
---------------
1. Add tests for auth module (coverage: 30%)
2. Remove hardcoded secrets (2 found)
3. Reduce function length in user.rs (3 functions > 50 lines)
4. Add error context for unwrap() calls (4 instances)
```

#### JSON Output

```json
{
  "summary": {
    "files_changed": 5,
    "lines_added": 234,
    "lines_deleted": 87,
    "issues_count": 12,
    "critical_count": 1,
    "high_count": 3,
    "medium_count": 5,
    "low_count": 3
  },
  "issues": [
    {
      "severity": "Critical",
      "category": "Security",
      "file_path": "src/auth.rs",
      "line_start": 45,
      "line_end": 45,
      "message": "Hardcoded API key detected",
      "suggestion": "Use environment variable",
      "code_snippet": "let api_key = \"sk-1234567890\";"
    }
  ],
  "quality_score": {
    "total": 72.0,
    "complexity": 68.0,
    "readability": 85.0,
    "testing": 60.0,
    "documentation": 75.0
  },
  "recommendations": [
    "Add tests for auth module (coverage: 30%)",
    "Remove hardcoded secrets (2 found)"
  ],
  "diff_stats": {
    "files_changed": 5,
    "insertions": 234,
    "deletions": 87
  }
}
```

## Usage Examples

### Example 1: Before Committing

```bash
# Stage your changes
git add src/auth.rs

# Review before committing
ferroclaw review --scope staged

# If review passes, commit
ferroclaw commit --yes
```

### Example 2: Review Feature Branch

```bash
# Review all changes in feature branch
ferroclaw review --scope main..feature-branch

# Show only critical and high issues
ferroclaw review --scope main..feature-branch --severity high
```

### Example 3: Review Specific Files

```bash
# Review only Rust files
ferroclaw review --pattern "**/*.rs"

# Review only documentation
ferroclaw review --pattern "**/*.md"
```

### Example 4: Automated Workflow

```bash
# Review staged changes
ferroclaw review --output json > review.json

# Check if critical issues exist
if jq -e '.summary.critical_count > 0' review.json; then
    echo "Critical issues found!"
    exit 1
fi

# Commit if no critical issues
ferroclaw commit --yes
```

## Integration with Git

### Pre-Commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash

# Run review on staged changes
ferroclaw review --scope staged --severity critical

# Exit with error if critical issues found
if [ $? -ne 0 ]; then
    echo "Commit blocked: Critical issues found"
    exit 1
fi
```

### CI Integration

`.github/workflows/review.yml`:

```yaml
name: Code Review

on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Ferroclaw
        run: cargo install ferroclaw
      - name: Run Review
        run: ferroclaw review --scope main..HEAD --output json > review.json
      - name: Upload Results
        uses: actions/upload-artifact@v2
        with:
          name: review-results
          path: review.json
```

## Best Practices

### 1. Review Before Committing

```bash
# Always review staged changes
git add .
ferroclaw review --scope staged
ferroclaw commit
```

### 2. Use Appropriate Scopes

```bash
# Before committing → staged
ferroclaw review --scope staged

# After editing → working
ferroclaw review --scope working

# Before merging → commit range
ferroclaw review --scope main..HEAD
```

### 3. Filter by Severity

```bash
# Focus on critical issues
ferroclaw review --severity critical

# Include all issues
ferroclaw review --severity low
```

### 4. Use File Patterns

```bash
# Review specific module
ferroclaw review --pattern "src/auth/**"

# Review tests
ferroclaw review --pattern "**/*_test.rs"
```

### 5. Automate with JSON Output

```bash
# Parse review results in scripts
RESULT=$(ferroclaw review --output json)
CRITICAL=$(echo $RESULT | jq '.summary.critical_count')

if [ "$CRITICAL" -gt 0 ]; then
    echo "Critical issues found!"
    exit 1
fi
```

## Troubleshooting

### Issue: "No staged changes found"

**Cause**: No files staged for commit

**Solution**:
```bash
# Stage files first
git add <files>

# Or stage all
git add .

# Then review
ferroclaw review --scope staged
```

### Issue: "Not a Git repository"

**Cause**: Not in a Git repository

**Solution**:
```bash
# Initialize repository
git init

# Or specify repository path
ferroclaw review --repo-path /path/to/repo
```

### Issue: Review fails on merge conflicts

**Cause**: Unresolved merge conflicts

**Solution**:
```bash
# Resolve conflicts first
git mergetool

# Stage resolved files
git add <resolved-files>

# Review again
ferroclaw review --scope staged
```

## See Also

- **FileEditTool**: Edit files before committing
- **GrepTool**: Search code during review
- **TaskSystem**: Track review tasks
