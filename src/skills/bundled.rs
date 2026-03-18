//! Bundled skill definitions — 84 skills across 16 categories.
//!
//! Each skill is a bash-type tool that delegates to common CLI commands.
//! The agent's LLM selects the right skill and provides arguments.

use crate::skills::manifest::{bash_skill, Param, SkillCategory, SkillManifest};
use crate::types::Capability;

/// Returns all bundled skill manifests.
pub fn bundled_skills() -> Vec<SkillManifest> {
    let mut skills = Vec::with_capacity(84);

    skills.extend(filesystem_skills());
    skills.extend(version_control_skills());
    skills.extend(code_analysis_skills());
    skills.extend(web_skills());
    skills.extend(database_skills());
    skills.extend(docker_skills());
    skills.extend(kubernetes_skills());
    skills.extend(system_skills());
    skills.extend(text_processing_skills());
    skills.extend(network_skills());
    skills.extend(security_skills());
    skills.extend(documentation_skills());
    skills.extend(testing_skills());
    skills.extend(package_management_skills());
    skills.extend(cloud_skills());
    skills.extend(media_skills());

    skills
}

// ── Filesystem (6) ──────────────────────────────────────────────────────────

fn filesystem_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "find_files",
            "Find files matching a glob pattern recursively",
            SkillCategory::Filesystem,
            &["find", "search", "glob"],
            "find {{path}} -name '{{pattern}}' {{?extra_args}} 2>/dev/null | head -200",
            &[
                Param::required("path", "Directory to search in"),
                Param::required("pattern", "File name pattern (glob)"),
                Param::optional("extra_args", "Additional find flags (e.g. -type f)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "tree_view",
            "Display directory tree structure",
            SkillCategory::Filesystem,
            &["tree", "directory", "structure"],
            "find {{path}} -maxdepth {{?depth}} -print 2>/dev/null | head -500 | sed 's|[^/]*/|  |g'",
            &[
                Param::required("path", "Root directory to display"),
                Param::optional("depth", "Maximum depth (default: 4)").with_default("4"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "file_info",
            "Show file metadata: size, permissions, timestamps",
            SkillCategory::Filesystem,
            &["stat", "metadata", "info"],
            "stat {{path}} 2>/dev/null || ls -la {{path}}",
            &[Param::required("path", "File or directory path")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "copy_file",
            "Copy a file or directory",
            SkillCategory::Filesystem,
            &["copy", "cp", "duplicate"],
            "cp {{?flags}} '{{source}}' '{{destination}}'",
            &[
                Param::required("source", "Source path"),
                Param::required("destination", "Destination path"),
                Param::optional("flags", "Copy flags (e.g. -r for recursive)"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
        bash_skill(
            "move_file",
            "Move or rename a file or directory",
            SkillCategory::Filesystem,
            &["move", "mv", "rename"],
            "mv {{?flags}} '{{source}}' '{{destination}}'",
            &[
                Param::required("source", "Source path"),
                Param::required("destination", "Destination path"),
                Param::optional("flags", "Move flags (e.g. -n for no-clobber)"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
        bash_skill(
            "tail_file",
            "Show the last N lines of a file",
            SkillCategory::Filesystem,
            &["tail", "last", "end"],
            "tail -n {{?lines}} '{{path}}'",
            &[
                Param::required("path", "File path to tail"),
                Param::optional("lines", "Number of lines (default: 50)").with_default("50"),
            ],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Version Control (8) ─────────────────────────────────────────────────────

fn version_control_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "git_status",
            "Show working tree status and staged changes",
            SkillCategory::VersionControl,
            &["git", "status", "changes"],
            "cd '{{path}}' && git status {{?flags}}",
            &[
                Param::required("path", "Repository path"),
                Param::optional("flags", "Additional flags (e.g. --short)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "git_diff",
            "Show file differences (staged or unstaged)",
            SkillCategory::VersionControl,
            &["git", "diff", "changes"],
            "cd '{{path}}' && git diff {{?flags}} {{?file}}",
            &[
                Param::required("path", "Repository path"),
                Param::optional("flags", "Diff flags (e.g. --staged, --stat)"),
                Param::optional("file", "Specific file to diff"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "git_log",
            "Show commit history with formatting options",
            SkillCategory::VersionControl,
            &["git", "log", "history", "commits"],
            "cd '{{path}}' && git log {{?flags}} -n {{?count}}",
            &[
                Param::required("path", "Repository path"),
                Param::optional("flags", "Log flags (e.g. --oneline, --graph)").with_default("--oneline --graph"),
                Param::optional("count", "Number of commits to show").with_default("20"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "git_commit",
            "Stage and commit changes with a message",
            SkillCategory::VersionControl,
            &["git", "commit", "save"],
            "cd '{{path}}' && git add {{files}} && git commit -m '{{message}}'",
            &[
                Param::required("path", "Repository path"),
                Param::required("files", "Files to stage (space-separated, or '.' for all)"),
                Param::required("message", "Commit message"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
        bash_skill(
            "git_branch",
            "List, create, or delete branches",
            SkillCategory::VersionControl,
            &["git", "branch", "branches"],
            "cd '{{path}}' && git branch {{?flags}} {{?name}}",
            &[
                Param::required("path", "Repository path"),
                Param::optional("flags", "Branch flags (e.g. -a for all, -d to delete)"),
                Param::optional("name", "Branch name (for create/delete)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "git_checkout",
            "Switch branches or restore working tree files",
            SkillCategory::VersionControl,
            &["git", "checkout", "switch"],
            "cd '{{path}}' && git checkout {{target}} {{?flags}}",
            &[
                Param::required("path", "Repository path"),
                Param::required("target", "Branch name, tag, or commit hash"),
                Param::optional("flags", "Checkout flags (e.g. -b to create new branch)"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
        bash_skill(
            "git_stash",
            "Stash or restore uncommitted changes",
            SkillCategory::VersionControl,
            &["git", "stash", "save", "restore"],
            "cd '{{path}}' && git stash {{?action}} {{?flags}}",
            &[
                Param::required("path", "Repository path"),
                Param::optional("action", "Stash action: push, pop, list, show, drop").with_default("push"),
                Param::optional("flags", "Additional flags (e.g. -m 'message')"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "git_blame",
            "Show line-by-line attribution for a file",
            SkillCategory::VersionControl,
            &["git", "blame", "annotate", "attribution"],
            "cd '{{path}}' && git blame {{?flags}} '{{file}}'",
            &[
                Param::required("path", "Repository path"),
                Param::required("file", "File to blame"),
                Param::optional("flags", "Blame flags (e.g. -L 10,20 for line range)"),
            ],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Code Analysis (6) ───────────────────────────────────────────────────────

fn code_analysis_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "grep_code",
            "Search code files for a pattern using ripgrep or grep",
            SkillCategory::CodeAnalysis,
            &["grep", "search", "find", "regex"],
            "cd '{{path}}' && (rg {{?flags}} '{{pattern}}' {{?glob}} 2>/dev/null || grep -rn '{{pattern}}' {{?glob}} .) | head -200",
            &[
                Param::required("path", "Directory to search"),
                Param::required("pattern", "Search pattern (regex)"),
                Param::optional("glob", "File glob filter (e.g. '*.rs', '*.py')"),
                Param::optional("flags", "Additional flags (e.g. -i for case-insensitive)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "count_lines",
            "Count lines of code by file type",
            SkillCategory::CodeAnalysis,
            &["loc", "sloc", "count", "lines"],
            "cd '{{path}}' && (tokei {{?flags}} 2>/dev/null || find . -name '{{?pattern}}' -exec wc -l {} + 2>/dev/null | tail -1)",
            &[
                Param::required("path", "Directory to analyze"),
                Param::optional("pattern", "File pattern to count (e.g. '*.rs')").with_default("*"),
                Param::optional("flags", "Additional flags"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "find_definition",
            "Find function, class, or type definitions in code",
            SkillCategory::CodeAnalysis,
            &["definition", "function", "class", "symbol"],
            "cd '{{path}}' && (rg '(fn |def |class |interface |struct |enum |type |const |let |var ){{symbol}}' {{?glob}} 2>/dev/null || grep -rn '{{symbol}}' . --include='{{?glob}}') | head -50",
            &[
                Param::required("path", "Directory to search"),
                Param::required("symbol", "Symbol name to find"),
                Param::optional("glob", "File filter (e.g. '*.rs')"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "find_references",
            "Find all references to a symbol across the codebase",
            SkillCategory::CodeAnalysis,
            &["references", "usages", "callers"],
            "cd '{{path}}' && (rg -n '{{symbol}}' {{?glob}} 2>/dev/null || grep -rn '{{symbol}}' . --include='{{?glob}}') | head -200",
            &[
                Param::required("path", "Directory to search"),
                Param::required("symbol", "Symbol to find references for"),
                Param::optional("glob", "File filter (e.g. '*.ts')"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "lint_check",
            "Run the appropriate linter for the project",
            SkillCategory::CodeAnalysis,
            &["lint", "check", "static analysis"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo clippy {{?flags}} 2>&1 || test -f package.json && npx eslint {{?target}} 2>&1 || test -f pyproject.toml && ruff check {{?target}} 2>&1 || echo 'No recognized linter config found') | head -200",
            &[
                Param::required("path", "Project directory"),
                Param::optional("target", "Specific file or directory to lint"),
                Param::optional("flags", "Additional linter flags"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "code_complexity",
            "Analyze code complexity metrics",
            SkillCategory::CodeAnalysis,
            &["complexity", "metrics", "quality"],
            "cd '{{path}}' && wc -l {{?glob}} | sort -rn | head -30",
            &[
                Param::required("path", "Directory to analyze"),
                Param::optional("glob", "File pattern to analyze").with_default("$(find . -name '*.rs' -o -name '*.py' -o -name '*.ts' -o -name '*.go' | head -100)"),
            ],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Web (5) ─────────────────────────────────────────────────────────────────

fn web_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "http_get",
            "Make an HTTP GET request and return the response",
            SkillCategory::Web,
            &["http", "get", "request", "api"],
            "curl -sS {{?flags}} '{{url}}'",
            &[
                Param::required("url", "URL to fetch"),
                Param::optional("flags", "Curl flags (e.g. -H 'Accept: application/json' -v)"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "http_post",
            "Make an HTTP POST request with a body",
            SkillCategory::Web,
            &["http", "post", "request", "api"],
            "curl -sS -X POST {{?flags}} -d '{{body}}' '{{url}}'",
            &[
                Param::required("url", "URL to post to"),
                Param::required("body", "Request body (JSON string)"),
                Param::optional("flags", "Curl flags (e.g. -H 'Content-Type: application/json')").with_default("-H 'Content-Type: application/json'"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "url_encode",
            "URL-encode or decode a string",
            SkillCategory::Web,
            &["url", "encode", "decode", "percent"],
            "python3 -c \"import urllib.parse; print(urllib.parse.{{action}}('{{text}}'))\"",
            &[
                Param::required("text", "Text to encode or decode"),
                Param::optional("action", "Action: quote or unquote").with_default("quote"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "download_file",
            "Download a file from a URL",
            SkillCategory::Web,
            &["download", "fetch", "save"],
            "curl -sSL {{?flags}} -o '{{output}}' '{{url}}'",
            &[
                Param::required("url", "URL to download"),
                Param::required("output", "Output file path"),
                Param::optional("flags", "Additional curl flags"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound, Capability::FsWrite],
        ),
        bash_skill(
            "check_url",
            "Check if a URL is reachable and return status code",
            SkillCategory::Web,
            &["check", "ping", "health", "status"],
            "curl -sS -o /dev/null -w 'HTTP %{{http_code}} | Time: %{{time_total}}s | Size: %{{size_download}} bytes' '{{url}}'",
            &[Param::required("url", "URL to check")],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
    ]
}

// ── Database (5) ────────────────────────────────────────────────────────────

fn database_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "sqlite_query",
            "Execute a SQL query against a SQLite database",
            SkillCategory::Database,
            &["sqlite", "sql", "query", "database"],
            "sqlite3 -header -column '{{db_path}}' '{{query}}'",
            &[
                Param::required("db_path", "Path to SQLite database file"),
                Param::required("query", "SQL query to execute"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "pg_query",
            "Execute a SQL query against a PostgreSQL database",
            SkillCategory::Database,
            &["postgres", "postgresql", "sql", "query"],
            "psql '{{connection_string}}' -c '{{query}}'",
            &[
                Param::required("connection_string", "PostgreSQL connection string"),
                Param::required("query", "SQL query to execute"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "db_tables",
            "List all tables in a SQLite database",
            SkillCategory::Database,
            &["tables", "schema", "list"],
            "sqlite3 '{{db_path}}' '.tables'",
            &[Param::required("db_path", "Path to SQLite database")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "db_schema",
            "Show the schema of a table in a SQLite database",
            SkillCategory::Database,
            &["schema", "describe", "columns"],
            "sqlite3 '{{db_path}}' '.schema {{table}}'",
            &[
                Param::required("db_path", "Path to SQLite database"),
                Param::required("table", "Table name"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "csv_to_sql",
            "Import a CSV file into a SQLite database table",
            SkillCategory::Database,
            &["import", "csv", "load"],
            "sqlite3 '{{db_path}}' '.mode csv' '.import {{csv_path}} {{table}}'",
            &[
                Param::required("db_path", "Path to SQLite database"),
                Param::required("csv_path", "Path to CSV file"),
                Param::required("table", "Target table name"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
    ]
}

// ── Docker (6) ──────────────────────────────────────────────────────────────

fn docker_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "docker_ps",
            "List running Docker containers",
            SkillCategory::Docker,
            &["docker", "containers", "list", "running"],
            "docker ps {{?flags}}",
            &[Param::optional("flags", "Additional flags (e.g. -a for all, --format)")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "docker_logs",
            "View container logs",
            SkillCategory::Docker,
            &["docker", "logs", "output"],
            "docker logs {{?flags}} '{{container}}'",
            &[
                Param::required("container", "Container name or ID"),
                Param::optional("flags", "Log flags (e.g. --tail 100, -f for follow)").with_default("--tail 100"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "docker_exec",
            "Execute a command inside a running container",
            SkillCategory::Docker,
            &["docker", "exec", "run", "shell"],
            "docker exec {{?flags}} '{{container}}' {{command}}",
            &[
                Param::required("container", "Container name or ID"),
                Param::required("command", "Command to execute"),
                Param::optional("flags", "Exec flags (e.g. -it for interactive)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "docker_build",
            "Build a Docker image from a Dockerfile",
            SkillCategory::Docker,
            &["docker", "build", "image"],
            "docker build {{?flags}} -t '{{tag}}' '{{path}}'",
            &[
                Param::required("path", "Build context directory"),
                Param::required("tag", "Image tag (e.g. myapp:latest)"),
                Param::optional("flags", "Build flags (e.g. --no-cache, --platform linux/amd64)"),
            ],
            &[Capability::ProcessExec, Capability::FsRead],
        ),
        bash_skill(
            "docker_images",
            "List Docker images",
            SkillCategory::Docker,
            &["docker", "images", "list"],
            "docker images {{?flags}}",
            &[Param::optional("flags", "Additional flags (e.g. --filter, --format)")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "docker_compose_up",
            "Start Docker Compose services",
            SkillCategory::Docker,
            &["docker", "compose", "up", "start"],
            "cd '{{path}}' && docker compose up {{?flags}}",
            &[
                Param::required("path", "Directory with docker-compose.yml"),
                Param::optional("flags", "Compose flags (e.g. -d for detached, --build)").with_default("-d"),
            ],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Kubernetes (5) ──────────────────────────────────────────────────────────

fn kubernetes_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "kubectl_get",
            "Get Kubernetes resources",
            SkillCategory::Kubernetes,
            &["k8s", "kubernetes", "get", "list"],
            "kubectl get {{resource}} {{?flags}} {{?name}}",
            &[
                Param::required("resource", "Resource type (pods, services, deployments, etc.)"),
                Param::optional("name", "Specific resource name"),
                Param::optional("flags", "Additional flags (e.g. -n namespace, -o yaml)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "kubectl_describe",
            "Describe a Kubernetes resource in detail",
            SkillCategory::Kubernetes,
            &["k8s", "kubernetes", "describe", "detail"],
            "kubectl describe {{resource}} {{name}} {{?flags}}",
            &[
                Param::required("resource", "Resource type"),
                Param::required("name", "Resource name"),
                Param::optional("flags", "Additional flags (e.g. -n namespace)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "kubectl_logs",
            "View logs from a Kubernetes pod",
            SkillCategory::Kubernetes,
            &["k8s", "kubernetes", "logs", "pod"],
            "kubectl logs {{pod}} {{?flags}}",
            &[
                Param::required("pod", "Pod name"),
                Param::optional("flags", "Log flags (e.g. --tail 100, -c container, -f)").with_default("--tail 100"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "kubectl_apply",
            "Apply a Kubernetes manifest file",
            SkillCategory::Kubernetes,
            &["k8s", "kubernetes", "apply", "deploy"],
            "kubectl apply -f '{{file}}' {{?flags}}",
            &[
                Param::required("file", "Path to manifest file or directory"),
                Param::optional("flags", "Additional flags (e.g. -n namespace, --dry-run=client)"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "kubectl_port_forward",
            "Forward a local port to a pod or service",
            SkillCategory::Kubernetes,
            &["k8s", "kubernetes", "port-forward", "proxy"],
            "kubectl port-forward {{resource}} {{ports}} {{?flags}} &",
            &[
                Param::required("resource", "Resource (e.g. pod/mypod, svc/myservice)"),
                Param::required("ports", "Port mapping (e.g. 8080:80)"),
                Param::optional("flags", "Additional flags (e.g. -n namespace)"),
            ],
            &[Capability::ProcessExec, Capability::NetListen],
        ),
    ]
}

// ── System (6) ──────────────────────────────────────────────────────────────

fn system_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "process_list",
            "List running processes with filtering",
            SkillCategory::System,
            &["ps", "process", "list", "running"],
            "ps aux {{?flags}} | head -50",
            &[Param::optional("flags", "Filter flags (pipe to grep, e.g. '| grep node')")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "system_info",
            "Display system information (OS, CPU, memory)",
            SkillCategory::System,
            &["system", "info", "hardware", "os"],
            "uname -a && echo '---' && (sysctl -n hw.memsize 2>/dev/null || free -h 2>/dev/null || echo 'Memory info unavailable') && echo '---' && (sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo 'CPU count unavailable')",
            &[],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "disk_usage",
            "Show disk usage for a path or filesystem",
            SkillCategory::System,
            &["disk", "usage", "space", "du", "df"],
            "du -sh {{?path}} {{?flags}} 2>/dev/null | sort -rh | head -30",
            &[
                Param::optional("path", "Path to analyze (default: current directory)").with_default("./*"),
                Param::optional("flags", "Additional flags"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "env_var",
            "Get the value of an environment variable",
            SkillCategory::System,
            &["env", "environment", "variable"],
            "printenv {{name}}",
            &[Param::required("name", "Environment variable name")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "which_command",
            "Find the location of a command",
            SkillCategory::System,
            &["which", "where", "find", "command"],
            "which {{command}} 2>/dev/null && {{command}} --version 2>/dev/null | head -3",
            &[Param::required("command", "Command name to locate")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "uptime_info",
            "Show system uptime and load average",
            SkillCategory::System,
            &["uptime", "load", "system"],
            "uptime",
            &[],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Text Processing (5) ────────────────────────────────────────────────────

fn text_processing_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "json_query",
            "Query JSON data using jq expressions",
            SkillCategory::TextProcessing,
            &["json", "jq", "query", "parse"],
            "echo '{{json}}' | jq '{{expression}}'",
            &[
                Param::required("json", "JSON string to query"),
                Param::required("expression", "jq expression (e.g. '.data[] | .name')"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "json_file_query",
            "Query a JSON file using jq expressions",
            SkillCategory::TextProcessing,
            &["json", "jq", "file", "query"],
            "jq '{{expression}}' '{{path}}'",
            &[
                Param::required("path", "Path to JSON file"),
                Param::required("expression", "jq expression"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "yaml_to_json",
            "Convert YAML to JSON",
            SkillCategory::TextProcessing,
            &["yaml", "json", "convert"],
            "python3 -c \"import sys, yaml, json; print(json.dumps(yaml.safe_load(open('{{path}}')), indent=2))\"",
            &[Param::required("path", "Path to YAML file")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "regex_match",
            "Match a regex pattern against input text and show matches",
            SkillCategory::TextProcessing,
            &["regex", "match", "pattern", "extract"],
            "echo '{{text}}' | grep -oE '{{pattern}}'",
            &[
                Param::required("text", "Input text"),
                Param::required("pattern", "Regex pattern to match"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "text_replace",
            "Find and replace text in files using sed",
            SkillCategory::TextProcessing,
            &["replace", "sed", "substitute", "find-replace"],
            "sed -i{{?backup}} 's/{{find}}/{{replace}}/g' '{{path}}'",
            &[
                Param::required("path", "File path"),
                Param::required("find", "Pattern to find (regex)"),
                Param::required("replace", "Replacement text"),
                Param::optional("backup", "Backup suffix (e.g. '.bak')"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
    ]
}

// ── Network (5) ─────────────────────────────────────────────────────────────

fn network_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "ping_host",
            "Ping a host to check connectivity",
            SkillCategory::Network,
            &["ping", "network", "connectivity"],
            "ping -c {{?count}} {{host}}",
            &[
                Param::required("host", "Hostname or IP address"),
                Param::optional("count", "Number of pings").with_default("4"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "port_check",
            "Check if a TCP port is open on a host",
            SkillCategory::Network,
            &["port", "check", "open", "tcp"],
            "(echo >/dev/tcp/{{host}}/{{port}}) 2>/dev/null && echo 'Port {{port}} on {{host}} is OPEN' || echo 'Port {{port}} on {{host}} is CLOSED'",
            &[
                Param::required("host", "Hostname or IP"),
                Param::required("port", "Port number"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "curl_request",
            "Make a curl request with custom headers and method",
            SkillCategory::Network,
            &["curl", "http", "request", "headers"],
            "curl -sS -X {{?method}} {{?headers}} {{?flags}} '{{url}}'",
            &[
                Param::required("url", "URL to request"),
                Param::optional("method", "HTTP method").with_default("GET"),
                Param::optional("headers", "Headers (e.g. -H 'Authorization: Bearer xxx')"),
                Param::optional("flags", "Additional curl flags (e.g. -d for body, -v for verbose)"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "dns_lookup",
            "Perform DNS lookup for a hostname",
            SkillCategory::Network,
            &["dns", "lookup", "resolve", "nslookup"],
            "(dig +short {{host}} {{?record_type}} 2>/dev/null || nslookup {{host}} 2>/dev/null || host {{host}} 2>/dev/null)",
            &[
                Param::required("host", "Hostname to look up"),
                Param::optional("record_type", "DNS record type (A, AAAA, MX, NS, TXT)").with_default("A"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "local_ip",
            "Show local network interface IP addresses",
            SkillCategory::Network,
            &["ip", "network", "interface", "address"],
            "(ifconfig 2>/dev/null || ip addr 2>/dev/null) | grep -E 'inet[^6]' | awk '{print $2}'",
            &[],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Security (5) ────────────────────────────────────────────────────────────

fn security_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "hash_file",
            "Compute SHA256 or other hash of a file",
            SkillCategory::Security,
            &["hash", "sha256", "checksum", "verify"],
            "(shasum -a {{?algorithm}} '{{path}}' 2>/dev/null || sha{{?algorithm}}sum '{{path}}')",
            &[
                Param::required("path", "File path to hash"),
                Param::optional("algorithm", "Hash algorithm (256, 512, 1)").with_default("256"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "check_permissions",
            "Check file permissions and ownership",
            SkillCategory::Security,
            &["permissions", "chmod", "ownership", "access"],
            "ls -la '{{path}}' && echo '---' && stat -c '%a %U:%G %n' '{{path}}' 2>/dev/null || stat -f '%Lp %Su:%Sg %N' '{{path}}'",
            &[Param::required("path", "File or directory path")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "scan_secrets",
            "Scan files for potential secrets, API keys, and tokens",
            SkillCategory::Security,
            &["secrets", "scan", "leak", "credentials"],
            "cd '{{path}}' && grep -rnE '(api[_-]?key|secret|password|token|credential|private[_-]?key)\\s*[:=]' --include='*.{{?ext}}' . 2>/dev/null | grep -v node_modules | grep -v '.git/' | head -50",
            &[
                Param::required("path", "Directory to scan"),
                Param::optional("ext", "File extension to filter (e.g. env, py, js)").with_default("*"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "generate_password",
            "Generate a cryptographically secure random password",
            SkillCategory::Security,
            &["password", "random", "generate", "secret"],
            "openssl rand -base64 {{?length}} | head -c {{?chars}} && echo",
            &[
                Param::optional("length", "Raw byte length for base64 encoding").with_default("32"),
                Param::optional("chars", "Maximum character count in output").with_default("44"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "encode_base64",
            "Base64 encode or decode a string",
            SkillCategory::Security,
            &["base64", "encode", "decode"],
            "echo -n '{{text}}' | base64 {{?flags}}",
            &[
                Param::required("text", "Text to encode/decode"),
                Param::optional("flags", "Flags (e.g. --decode or -d to decode)"),
            ],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Documentation (5) ──────────────────────────────────────────────────────

fn documentation_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "word_count",
            "Count words, lines, and characters in files",
            SkillCategory::Documentation,
            &["wc", "count", "words", "lines"],
            "wc {{?flags}} '{{path}}'",
            &[
                Param::required("path", "File or glob pattern"),
                Param::optional("flags", "wc flags (-w words, -l lines, -c bytes)").with_default("-lwc"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "markdown_toc",
            "Generate a table of contents from markdown headers",
            SkillCategory::Documentation,
            &["markdown", "toc", "table of contents"],
            "grep -E '^#{1,6} ' '{{path}}' | sed 's/^## /  - /; s/^### /    - /; s/^#### /      - /; s/^##### /        - /; s/^###### /          - /; s/^# /- /'",
            &[Param::required("path", "Path to markdown file")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "doc_links_check",
            "Check for broken links in markdown files",
            SkillCategory::Documentation,
            &["links", "broken", "check", "markdown"],
            "cd '{{path}}' && grep -rhoE '\\[.*?\\]\\(([^)]+)\\)' --include='*.md' . | grep -oE '\\(([^)]+)\\)' | tr -d '()' | while read -r link; do if echo \"$link\" | grep -qE '^https?://'; then echo \"[URL] $link\"; elif [ ! -e \"$link\" ]; then echo \"[BROKEN] $link\"; fi; done | head -50",
            &[Param::required("path", "Directory to check")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "changelog_entry",
            "Generate a changelog entry from recent git commits",
            SkillCategory::Documentation,
            &["changelog", "release", "notes"],
            "cd '{{path}}' && echo '## {{?version}}' && echo '' && git log --oneline {{?since}} | sed 's/^[a-f0-9]* /- /' | head -50",
            &[
                Param::required("path", "Repository path"),
                Param::optional("version", "Version label").with_default("Unreleased"),
                Param::optional("since", "Since ref (e.g. v1.0.0..HEAD)"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "readme_check",
            "Verify that a README exists and check its completeness",
            SkillCategory::Documentation,
            &["readme", "check", "verify"],
            "cd '{{path}}' && if [ -f README.md ]; then echo 'README.md found' && echo '---' && echo \"Lines: $(wc -l < README.md)\" && echo \"Words: $(wc -w < README.md)\" && echo '---' && echo 'Sections:' && grep -E '^#{1,3} ' README.md; else echo 'No README.md found'; fi",
            &[Param::required("path", "Project directory")],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Testing (5) ─────────────────────────────────────────────────────────────

fn testing_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "run_tests",
            "Run the project's test suite (auto-detects framework)",
            SkillCategory::Testing,
            &["test", "run", "suite"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo test {{?flags}} 2>&1 || test -f package.json && npm test {{?flags}} 2>&1 || test -f pyproject.toml && pytest {{?flags}} 2>&1 || test -f go.mod && go test ./... {{?flags}} 2>&1 || echo 'No recognized test framework found') | tail -50",
            &[
                Param::required("path", "Project directory"),
                Param::optional("flags", "Additional test flags"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "test_coverage",
            "Generate test coverage report",
            SkillCategory::Testing,
            &["coverage", "report", "test"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo tarpaulin --out Stdout 2>&1 || test -f package.json && npx jest --coverage 2>&1 || test -f pyproject.toml && pytest --cov={{?module}} 2>&1 || test -f go.mod && go test -cover ./... 2>&1 || echo 'No coverage tool detected') | tail -30",
            &[
                Param::required("path", "Project directory"),
                Param::optional("module", "Module to measure coverage for"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "run_benchmarks",
            "Run performance benchmarks",
            SkillCategory::Testing,
            &["benchmark", "perf", "performance"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo bench {{?flags}} 2>&1 || test -f go.mod && go test -bench='{{?pattern}}' -benchmem ./... 2>&1 || echo 'No benchmark framework found') | tail -50",
            &[
                Param::required("path", "Project directory"),
                Param::optional("flags", "Benchmark flags"),
                Param::optional("pattern", "Benchmark pattern to match").with_default("."),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "test_single",
            "Run a single test by name",
            SkillCategory::Testing,
            &["test", "single", "specific"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo test '{{test_name}}' {{?flags}} 2>&1 || test -f package.json && npx jest -t '{{test_name}}' 2>&1 || test -f pyproject.toml && pytest -k '{{test_name}}' 2>&1 || test -f go.mod && go test -run '{{test_name}}' ./... 2>&1) | tail -30",
            &[
                Param::required("path", "Project directory"),
                Param::required("test_name", "Test name or pattern"),
                Param::optional("flags", "Additional flags"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "test_watch",
            "Watch for file changes and re-run tests",
            SkillCategory::Testing,
            &["watch", "test", "auto"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo watch -x test 2>&1 || test -f package.json && npx jest --watch 2>&1 || test -f pyproject.toml && ptw 2>&1 || echo 'No watch mode available')",
            &[
                Param::required("path", "Project directory"),
            ],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Package Management (5) ──────────────────────────────────────────────────

fn package_management_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "npm_list",
            "List installed npm packages and their versions",
            SkillCategory::PackageManagement,
            &["npm", "node", "packages", "dependencies"],
            "cd '{{path}}' && npm list {{?flags}}",
            &[
                Param::required("path", "Project directory"),
                Param::optional("flags", "List flags (e.g. --depth=0, --json)").with_default("--depth=0"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "pip_list",
            "List installed Python packages",
            SkillCategory::PackageManagement,
            &["pip", "python", "packages"],
            "pip list {{?flags}}",
            &[Param::optional("flags", "List flags (e.g. --outdated, --format=json)")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "cargo_deps",
            "List Rust crate dependencies",
            SkillCategory::PackageManagement,
            &["cargo", "rust", "crates", "dependencies"],
            "cd '{{path}}' && cargo tree {{?flags}}",
            &[
                Param::required("path", "Project directory"),
                Param::optional("flags", "Tree flags (e.g. --depth 1, -e features)").with_default("--depth 1"),
            ],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "outdated_check",
            "Check for outdated dependencies",
            SkillCategory::PackageManagement,
            &["outdated", "update", "upgrade"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo outdated 2>&1 || test -f package.json && npm outdated 2>&1 || test -f pyproject.toml && pip list --outdated 2>&1 || test -f go.mod && go list -u -m all 2>&1 || echo 'No package manager detected')",
            &[Param::required("path", "Project directory")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "license_check",
            "Check licenses of project dependencies",
            SkillCategory::PackageManagement,
            &["license", "legal", "compliance"],
            "cd '{{path}}' && (test -f Cargo.toml && cargo license 2>&1 || test -f package.json && npx license-checker --summary 2>&1 || echo 'No license checker available')",
            &[Param::required("path", "Project directory")],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Cloud (5) ───────────────────────────────────────────────────────────────

fn cloud_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "aws_s3_ls",
            "List AWS S3 buckets or objects",
            SkillCategory::Cloud,
            &["aws", "s3", "cloud", "storage"],
            "aws s3 ls {{?path}} {{?flags}}",
            &[
                Param::optional("path", "S3 path (e.g. s3://bucket/prefix/)"),
                Param::optional("flags", "Additional flags (e.g. --recursive, --human-readable)"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "env_check",
            "Check that required environment variables are set",
            SkillCategory::Cloud,
            &["env", "check", "validate", "config"],
            "for var in {{vars}}; do if [ -n \"$(printenv $var)\" ]; then echo \"OK: $var is set\"; else echo \"MISSING: $var\"; fi; done",
            &[Param::required("vars", "Space-separated list of variable names to check")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "terraform_plan",
            "Run Terraform plan to preview infrastructure changes",
            SkillCategory::Cloud,
            &["terraform", "infra", "iac", "plan"],
            "cd '{{path}}' && terraform plan {{?flags}}",
            &[
                Param::required("path", "Terraform project directory"),
                Param::optional("flags", "Plan flags (e.g. -var-file=prod.tfvars)"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "ssh_command",
            "Execute a command on a remote host via SSH",
            SkillCategory::Cloud,
            &["ssh", "remote", "execute"],
            "ssh {{?flags}} '{{host}}' '{{command}}'",
            &[
                Param::required("host", "SSH host (user@hostname)"),
                Param::required("command", "Command to execute remotely"),
                Param::optional("flags", "SSH flags (e.g. -i key.pem, -p 2222)"),
            ],
            &[Capability::ProcessExec, Capability::NetOutbound],
        ),
        bash_skill(
            "gcloud_info",
            "Show Google Cloud project and account info",
            SkillCategory::Cloud,
            &["gcp", "google", "cloud"],
            "gcloud info --format='value(config.project, config.account)' 2>/dev/null || echo 'gcloud CLI not installed'",
            &[],
            &[Capability::ProcessExec],
        ),
    ]
}

// ── Media (5) ───────────────────────────────────────────────────────────────

fn media_skills() -> Vec<SkillManifest> {
    vec![
        bash_skill(
            "image_info",
            "Show image metadata (dimensions, format, size)",
            SkillCategory::Media,
            &["image", "info", "metadata", "exif"],
            "(file '{{path}}' && echo '---' && (identify '{{path}}' 2>/dev/null || sips -g pixelWidth -g pixelHeight '{{path}}' 2>/dev/null || echo 'No image tool available'))",
            &[Param::required("path", "Path to image file")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "image_resize",
            "Resize an image to specified dimensions",
            SkillCategory::Media,
            &["image", "resize", "scale", "convert"],
            "(convert '{{input}}' -resize '{{size}}' '{{output}}' 2>/dev/null || sips --resampleWidth {{?width}} '{{input}}' --out '{{output}}' 2>/dev/null || echo 'No image resize tool (ImageMagick or sips) available')",
            &[
                Param::required("input", "Input image path"),
                Param::required("output", "Output image path"),
                Param::optional("size", "Target size (e.g. 800x600, 50%)").with_default("800x600"),
                Param::optional("width", "Target width in pixels for sips").with_default("800"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
        bash_skill(
            "pdf_text",
            "Extract text from a PDF file",
            SkillCategory::Media,
            &["pdf", "text", "extract", "ocr"],
            "(pdftotext '{{path}}' - 2>/dev/null || python3 -c \"import PyPDF2; r=PyPDF2.PdfReader('{{path}}'); print('\\n'.join(p.extract_text() or '' for p in r.pages))\" 2>/dev/null || echo 'No PDF text extraction tool available') | head -500",
            &[Param::required("path", "Path to PDF file")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "file_checksum",
            "Compute multiple checksums (MD5, SHA256) for a file",
            SkillCategory::Media,
            &["checksum", "hash", "verify", "integrity"],
            "echo 'MD5:' && (md5sum '{{path}}' 2>/dev/null || md5 '{{path}}') && echo 'SHA256:' && (sha256sum '{{path}}' 2>/dev/null || shasum -a 256 '{{path}}')",
            &[Param::required("path", "File path")],
            &[Capability::ProcessExec],
        ),
        bash_skill(
            "archive_create",
            "Create a compressed archive (tar.gz or zip)",
            SkillCategory::Media,
            &["archive", "compress", "tar", "zip"],
            "{{?format}} '{{output}}' {{?flags}} {{source}}",
            &[
                Param::required("source", "Source file or directory"),
                Param::required("output", "Output archive path"),
                Param::optional("format", "Archive command (tar czf or zip -r)").with_default("tar czf"),
                Param::optional("flags", "Additional flags"),
            ],
            &[Capability::ProcessExec, Capability::FsWrite],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_skill_count() {
        let skills = bundled_skills();
        assert!(
            skills.len() >= 70,
            "Expected 70+ bundled skills, got {}",
            skills.len()
        );
    }

    #[test]
    fn test_category_coverage() {
        let skills = bundled_skills();
        let categories: HashSet<SkillCategory> = skills.iter().map(|s| s.category).collect();
        assert_eq!(
            categories.len(),
            16,
            "Expected 16 categories, got {}",
            categories.len()
        );
        for cat in SkillCategory::all() {
            assert!(
                categories.contains(cat),
                "Missing category: {}",
                cat.display_name()
            );
        }
    }

    #[test]
    fn test_unique_names() {
        let skills = bundled_skills();
        let names: HashSet<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names.len(),
            skills.len(),
            "Duplicate skill names detected"
        );
    }

    #[test]
    fn test_all_skills_have_capabilities() {
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
    fn test_all_skills_have_description() {
        let skills = bundled_skills();
        for skill in &skills {
            assert!(
                !skill.description.is_empty(),
                "Skill '{}' has no description",
                skill.name
            );
        }
    }

    #[test]
    fn test_skills_per_category() {
        let skills = bundled_skills();
        let mut counts: std::collections::HashMap<SkillCategory, usize> =
            std::collections::HashMap::new();
        for skill in &skills {
            *counts.entry(skill.category).or_insert(0) += 1;
        }
        for (cat, count) in &counts {
            assert!(
                *count >= 4,
                "Category '{}' has only {} skills (need 4+)",
                cat.display_name(),
                count
            );
        }
    }
}
