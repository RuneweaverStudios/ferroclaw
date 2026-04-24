# Ferroclaw Tool Gap Analysis

**Date**: 2025-02-10
**Purpose**: Complete analysis of missing tools for Hermes-style agent capabilities

---

## Executive Summary

Ferroclaw has excellent tool coverage with 12 built-in tools and 84 bundled skills across 16 categories. However, to achieve complete Hermes-style agent harness capabilities, approximately **20-25 additional tools** are needed across 4 categories.

**Current Coverage**: ~70%
**Target Coverage**: ~95%+
**Estimated Implementation Time**: 4-6 weeks

---

## Analysis by Category

### Category 1: File System ✅ COMPLETE
**Current**: 3 built-in + 6 skills
**Missing**: 0 tools
**Coverage**: 100%

**Tools Available**:
- ✅ `read_file` - Read file contents
- ✅ `write_file` - Write to files
- ✅ `list_directory` - List directory entries
- ✅ `file_edit` - Exact string replacement
- ✅ `glob` - Pattern matching
- Skills: `find_files`, `tree_view`, `file_info`, `copy_file`, `move_file`, `tail_file`

**Priority**: None (complete)

---

### Category 2: Command Execution ⚠️ MOSTLY COMPLETE
**Current**: 1 built-in tool (`bash`)
**Missing**: 3-4 tools
**Coverage**: 25%

**Tools Available**:
- ✅ `bash` - Execute shell commands

**Missing Tools** (High Priority):

1. **`execute_code`**
   - **Description**: Run code in Python, Node.js, Rust, or other languages
   - **Input**: `language`, `code`, `timeout` (default: 30)
   - **Capability**: `process_exec`
   - **Use Cases**:
     - Run Python scripts
     - Execute JavaScript/Node.js code
     - Test Rust code snippets
     - Run one-liners in multiple languages
   - **Implementation Complexity**: Medium
   - **Dependencies**: `tokio` (async runtime)
   - **Estimated Time**: 1-2 days

2. **`start_process`**
   - **Description**: Start a long-running process and get its PID
   - **Input**: `command`, `working_dir`, `env_vars` (optional)
   - **Capability**: `process_exec`
   - **Use Cases**:
     - Start development servers
     - Run background processes
     - Start long-running computations
   - **Implementation Complexity**: Medium
   - **Dependencies**: Process tracking system
   - **Estimated Time**: 1-2 days

3. **`stop_process`**
   - **Description**: Stop a running process by PID or name
   - **Input**: `pid`, `process_name` (optional)
   - **Capability**: `process_exec`
   - **Use Cases**:
     - Stop development servers
     - Kill background processes
     - Clean up completed processes
   - **Implementation Complexity**: Medium
   - **Dependencies**: Process tracking system
   - **Estimated Time**: 1-2 days

4. **`stream_output`**
   - **Description**: Stream output from a running command in real-time
   - **Input**: `pid`, `timeout` (optional)
   - **Capability**: `process_exec`
   - **Use Cases**:
     - Monitor long-running processes
     - Show progress to user
     - Stream logs from running services
   - **Implementation Complexity**: High
   - **Dependencies**: Process tracking + WebSocket or SSE
   - **Estimated Time**: 2-3 days

**Priority**: HIGH

---

### Category 3: Web & Network ✅ GOOD
**Current**: 1 built-in + 10 skills
**Missing**: 1 tool
**Coverage**: 92%

**Tools Available**:
- ✅ `web_fetch` - HTTP GET with size limits
- Skills: `http_get`, `http_post`, `download_file`, `check_url`, `url_encode`

**Missing Tools**:

1. **`web_search`** (LOW PRIORITY - MCP can handle)
   - **Description**: Search the web for information
   - **Input**: `query`, `num_results` (default: 10), `source` (optional)
   - **Capability**: `net_outbound`
   - **Use Cases**:
     - Get current information
     - Find documentation
     - Research topics
     - Answer factual questions
   - **Implementation Complexity**: Medium
   - **Dependencies**: `reqwest` (already have) + search API
   - **Alternative**: Use MCP server (e.g., Brave Search, Tavily)
   - **Estimated Time**: 2-3 days

**Priority**: LOW (MCP can handle)

---

### Category 4: Canvas/Workspace ✅ GOOD
**Current**: 3 built-in tools
**Missing**: 2 tools
**Coverage**: 60%

**Tools Available**:
- ✅ `canvas_list_modules` - List all tiles
- ✅ `canvas_create_tile` - Create new tiles
- ✅ `canvas_update_tile` - Modify existing tiles

**Missing Tools**:

1. **`canvas_link_modules`**
   - **Description**: Create connections/links between tiles
   - **Input**: `source_id`, `target_id`, `link_type` (optional)
   - **Capability**: `canvas_write`
   - **Use Cases**:
     - Connect related tiles
     - Create task dependencies
     - Visualize project structure
   - **Implementation Complexity**: Medium
   - **Dependencies**: Canvas API
   - **Estimated Time**: 1-2 days

2. **`canvas_search`**
   - **Description**: Search content across all tiles
   - **Input**: `query`, `filters` (optional)
   - **Capability**: `canvas_read`
   - **Use Cases**:
     - Find specific content
     - Search by metadata
     - Locate relevant tiles
   - **Implementation Complexity**: Medium
   - **Dependencies**: Canvas API + full-text search
   - **Estimated Time**: 1-2 days

**Priority**: MEDIUM

---

### Category 5: Code Intelligence ⚠️ PARTIAL
**Current**: 2 built-in + 6 skills
**Missing**: 5 tools
**Coverage**: 40%

**Tools Available**:
- ✅ `grep` - Search code with patterns
- Skills: `grep_code`, `count_lines`, `find_definition`, `find_references`, `lint_check`, `code_complexity`

**Missing Tools** (High Priority):

1. **`analyze_code`**
   - **Description**: Analyze code structure, dependencies, complexity
   - **Input**: `path`, `analysis_type` (optional, default: "all"), `language` (auto-detect)
   - **Capability**: `fs_read`
   - **Features**:
     - Multi-language support: Rust, Python, JavaScript, TypeScript
     - Analysis types: structure, dependencies, complexity, imports, all
     - Outputs: function/class detection, complexity metrics, code statistics
   - **Implementation Complexity**: Medium
   - **Dependencies**: Language parsers (can start with simple regex)
   - **Estimated Time**: 2-3 days

2. **`refactor_code`**
   - **Description**: Apply code refactorings
   - **Input**: `path`, `refactoring_type`, `target`, `new_name`, `lines` (optional)
   - **Capability**: `fs_read`, `fs_write`
   - **Refactorings**:
     - `extract_function` - Extract selected code into function
     - `inline_function` - Replace function call with function body
     - `rename` - Rename symbol across file
     - `extract_variable` - Extract expression into variable
     - `move_declaration` - Move declaration to another location
   - **Implementation Complexity**: Medium
   - **Dependencies**: Language-aware AST or regex
   - **Estimated Time**: 2-3 days

3. **`generate_tests`**
   - **Description**: Generate unit and integration tests for code
   - **Input**: `path`, `test_type` (unit/integration/both, default: "unit"), `framework` (auto-detect)
   - **Capability**: `fs_read`, `fs_write`
   - **Features**:
     - Multi-language support: Rust, Python, JavaScript, TypeScript
     - Framework detection: Jest, pytest, cargo test
     - Generates test files with proper structure
   - **Implementation Complexity**: Medium
   - **Dependencies**: Template system per language
   - **Estimated Time**: 2-3 days

4. **`review_code`**
   - **Description**: Perform automated code review with quality scoring
   - **Input**: `path`, `severity` (all/high/medium/low), `categories` (all/security/performance/style/correctness/complexity)
   - **Capability**: `fs_read`
   - **Features**:
     - Quality scoring: 0-100 scale
     - Severity levels: high, medium, low
     - Categories: security, performance, style, correctness, complexity
     - Provides actionable recommendations
   - **Implementation Complexity**: Medium
   - **Dependencies**: Static analysis rules + LLM for recommendations
   - **Estimated Time**: 2-3 days

5. **`find_bugs`**
   - **Description**: Find bugs and potential issues in code
   - **Input**: `path`, `bug_type` (all/security/logic/concurrency/memory/performance)
   - **Capability**: `fs_read`
   - **Features**:
     - Bug types: security, logic, concurrency, memory, performance
     - Pattern-based detection with severity classification
     - Supports multiple languages
   - **Implementation Complexity**: Medium
   - **Dependencies**: Pattern database per language
   - **Estimated Time**: 2-3 days

**Priority**: HIGH

---

### Category 6: Development Workflow ✅ GOOD
**Current**: 1 built-in + 15 skills
**Missing**: 3-4 tools
**Coverage**: 75%

**Tools Available**:
- ✅ `commit` - Git commit with conventional format
- Skills: `git_status`, `git_diff`, `git_log`, `git_commit`, `git_branch`, `git_checkout`, `git_stash`, `git_blame`

**Missing Tools** (Low Priority - Skills Exist):

1. **`build`** (SKILL EXISTS - PROMOTE TO BUILT-IN)
   - **Description**: Compile, bundle, or build projects
   - **Input**: `path` (optional, default: current directory), `target` (development/production/release/debug/test/clean/all), `tool` (auto/cargo/npm/yarn/pip/go/bundler/composer/make/cmake/gradle/mvn/dotnet), `clean` (default: false), `args` (optional), `output_path` (optional), `dry_run` (default: false), `verbose` (default: false)
   - **Capability**: `process_exec`
   - **Features**:
     - Multi-language support: Rust (cargo), Node.js (npm/yarn), Python (pip), Go (go), Ruby (bundler), PHP (composer)
     - Also supports: Make, CMake, Gradle, Maven, .NET
     - Auto-detects project type and build system
     - Targets: development, production, release, debug, test
     - Clean artifacts before building
     - Custom build args
     - Verbose output
   - **Implementation Complexity**: Medium
   - **Dependencies**: None (uses existing skills)
   - **Estimated Time**: 1-2 days
   - **Note**: Skill exists but should be promoted to built-in for better integration

2. **`install_deps`** (SKILL EXISTS)
   - **Description**: Install project dependencies
   - **Input**: `path` (optional), `package_manager` (auto/cargo/npm/yarn/pip/bundler/composer/go), `args` (optional)
   - **Capability**: `process_exec`
   - **Features**:
     - Auto-detect package manager
     - Install dependencies
     - Update lockfiles
   - **Implementation Complexity**: Medium
   - **Dependencies**: None (uses existing skills)
   - **Estimated Time**: 1-2 days

3. **`format_code`** (NEW TOOL)
   - **Description**: Format code with project formatter
   - **Input**: `path` (optional, default: current directory), `tool` (auto/prettier/black/rustfmt/gofmt/autopep8), `args` (optional)
   - **Capability**: `fs_read`, `fs_write`
   - **Features**:
     - Auto-detect formatter
     - Format files or directories
     - Supports multiple formatters
   - **Implementation Complexity**: Low
   - **Dependencies**: External formatter (optional)
   - **Estimated Time**: 1 day

**Priority**: LOW

**Note**: `build` and `install_deps` skills already exist and can be used immediately. Promoting to built-in would improve integration but is not critical.

---

### Category 7: Memory & Context ✅ COMPLETE
**Current**: 2 built-in + TaskSystem + MemdirSystem
**Missing**: 0 tools
**Coverage**: 100%

**Tools Available**:
- ✅ `memory_search` - Full-text search of memories
- ✅ `memory_store` - Store key-value memories
- TaskSystem - SQLite-backed task tracking with dependencies
- MemdirSystem - File-based persistent memory

**Priority**: None (complete)

---

### Category 8: Reasoning & Planning ⚠️ PARTIAL
**Current**: TaskSystem + PlanMode + AgentTool
**Missing**: 1-2 tools
**Coverage**: 50%

**Tools Available**:
- ✅ TaskSystem - Task management with dependencies and status workflow
- ✅ PlanMode - Structured 4-phase planning (Research, Planning, Implementation, Verification)
- ✅ AgentTool - Spawn specialized subagents with isolated context

**Missing Tools** (Low Priority):

1. **`evaluate_result`** (NEW TOOL - HIGH PRIORITY)
   - **Description**: Evaluate the result of an action against success criteria
   - **Input**: `task`, `result`, `success_criteria`, `metrics` (optional)
   - **Capability**: None
   - **Features**:
     - Success criteria evaluation
     - Metrics analysis (performance, quality, errors)
     - Detailed assessment with recommendations
     - Success rate calculation
     - Outputs: met/unmet criteria, success rate, recommendations
   - **Implementation Complexity**: Low
   - **Dependencies**: None
   - **Estimated Time**: 1 day

2-3. Other Tools (OPTIONAL):
   - `decompose_task` - Break down complex task into subtasks (LLM-based, not a separate tool)
   - `create_plan` - Create execution plan (LLM-based, not a separate tool)
   - `execute_plan` - Execute plan step-by-step (LLM-based, not a separate tool)

**Note**: Many of these are already handled by the TaskSystem and PlanMode, and creating them as separate tools may be redundant.

**Priority**: MEDIUM

---

### Category 9: Collaboration ❌ COMPLETELY MISSING
**Current**: 0 tools
**Missing**: 4 tools
**Coverage**: 0%

**Missing Tools** (High Priority):

1. **`notify_user`**
   - **Description**: Send notifications to user
   - **Input**: `message`, `level` (info/warning/error/success), `channel` (terminal/telegram/slack/email)
   - **Capability**: None
   - **Features**:
     - Multi-level notifications with icons
     - Multiple output channels
     - Formatted output
   - **Implementation Complexity**: Low
   - **Dependencies**: Channel adapters (can start with terminal only)
   - **Estimated Time**: 1 day

2. **`request_approval`**
   - **Description**: Request human approval before proceeding with an action
   - **Input**: `action`, `description`, `auto_approve` (default: false)
   - **Capability**: None
   - **Features**:
     - Interactive approval prompts
     - Auto-approval mode for automation
     - Outputs: approval/rejection status
     - Action confirmation
   - **Implementation Complexity**: Low-Medium
   - **Dependencies**: None (terminal I/O)
   - **Estimated Time**: 1-2 days

3. **`share_context`**
   - **Description**: Share workspace context (tasks, memory, canvas, or all)
   - **Input**: `context_type` (workspace/tasks/memory/canvas/all), `format` (text/json/markdown)
   - **Capability**: `memory_read`
   - **Features**:
     - Multiple context types
     - Multiple output formats
     - Structured context export
   - **Implementation Complexity**: Low
   - **Dependencies**: TaskSystem, MemdirSystem, Canvas API
   - **Estimated Time**: 1 day

4. **`comment`**
   - **Description**: Add comments or annotations to files or tiles
   - **Input**: `target` (file or tile), `target_type` (file/tile), `comment`, `line` (for file comments)
   - **Capability**: `fs_write` (for files), `canvas_write` (for tiles)
   - **Features**:
     - File and tile commenting
     - Line-specific comments
     - Annotation markers in files
   - **Implementation Complexity**: Low-Medium
   - **Dependencies**: File system, Canvas API
   - **Estimated Time**: 1 day

**Priority**: HIGH

---

### Category 10: Monitoring & Debugging ⚠️ PARTIAL
**Current**: HookSystem + Audit Log
**Missing**: 4 tools
**Coverage**: 30%

**Current Capabilities**:
- ✅ HookSystem - Event logging (6 lifecycle hooks)
- ✅ Audit Log - Hash-chained append-only log of all tool calls

**Missing Tools** (High Priority):

1. **`get_logs`**
   - **Description**: Retrieve execution logs with filtering
   - **Input**: `limit` (default: 100), `level` (all/error/warning/info/debug), `since` (optional - timestamp or duration)
   - **Capability**: `memory_read`
   - **Features**:
     - Log filtering by level
     - Time-based filtering with --since
     - Limit control with --limit
     - Formatted output
   - **Implementation Complexity**: Medium
   - **Dependencies**: Log storage system
   - **Estimated Time**: 1-2 days

2. **`trace_execution`**
   - **Description**: Trace tool call chains and execution history
   - **Input**: `execution_id` (optional), `format` (tree/timeline/table, default: tree)
   - **Capability**: `memory_read`
   - **Features**:
     - Multiple trace views (tree, timeline, table)
     - Tool call chain visualization
     - Execution statistics (count, duration, success rate)
     - Parent-child relationships
   - **Implementation Complexity**: Medium
   - **Dependencies**: Log storage + tracking system
   - **Estimated Time**: 2-3 days

3. **`measure_metrics`**
   - **Description**: Measure and report performance metrics for tools
   - **Input**: `tool` (optional - specific tool to measure), `category` (all/fast/slow/errors)
   - **Capability**: `memory_read`
   - **Features**:
     - Performance metrics: total calls, duration, success rate
     - Top N fastest/slowest tools
     - Top N most called tools
     - Error tracking and rates
     - Category-based analysis
   - **Implementation Complexity**: Medium
   - **Dependencies**: Metrics collection system
   - **Estimated Time**: 2-3 days

4. **`debug_session`**
   - **Description**: Interactive debugging session with breakpoints and inspection
   - **Input**: `execution_id` (optional), `breakpoint` (optional)
   - **Capability**: `memory_read`, `process_exec`
   - **Features**:
     - Interactive breakpoints
     - Step-through execution
     - Variable inspection
     - Call stack trace
   - **Implementation Complexity**: High
   - **Dependencies**: Debugging infrastructure
   - **Estimated Time**: 3-5 days

**Priority**: HIGH

---

## Implementation Recommendations

### Phase 1: High Priority (2-3 weeks)
**Goal**: Enable sophisticated code intelligence and collaboration tools

**Tools to Implement**:
1. Code Intelligence (5 tools): `analyze_code`, `refactor_code`, `generate_tests`, `review_code`, `find_bugs`
2. Collaboration (4 tools): `notify_user`, `request_approval`, `share_context`, `comment`
3. Reasoning (1 tool): `evaluate_result`

**Dependencies**: None (all existing)

**Estimated Time**: 2-3 weeks

### Phase 2: Medium Priority (1-2 weeks)
**Goal**: Add execution, monitoring, and workspace tools

**Tools to Implement**:
1. Command Execution (3 tools): `execute_code`, `start_process`, `stop_process`, `stream_output`
2. Monitoring (4 tools): `get_logs`, `trace_execution`, `measure_metrics`, `debug_session`
3. Canvas (2 tools): `canvas_link_modules`, `canvas_search`

**Dependencies**: Process tracking system for process management tools

**Estimated Time**: 1-2 weeks

### Phase 3: Low Priority (1 week, optional)
**Goal**: Polish and enhance existing functionality

**Tools to Implement**:
- Promote `build` skill to built-in
- Promote `install_deps` skill to built-in
- Implement `format_code` tool
- Optionally implement `web_search` (MCP can handle)

**Estimated Time**: 1 week

---

## Technical Considerations

### New Dependencies Required

```toml
[dependencies]
# Optional - for advanced code parsing (if needed for complex refactorings)
tree-sitter = "0.22"

# For process management (needed for start_process/stop_process)
# sysinfo = "0.30"  # For getting process information

# Optional - for monitoring tools
tracing-appender = "0.2"  # For log file rotation
opentelemetry = "0.23"  # Optional, for distributed tracing
```

### Capability Requirements

All new tools should properly require capabilities:

| Tool | Required Capabilities |
|-------|-------------------|
| `analyze_code` | `fs_read` |
| `refactor_code` | `fs_read`, `fs_write` |
| `generate_tests` | `fs_read`, `fs_write` |
| `review_code` | `fs_read` |
| `find_bugs` | `fs_read` |
| `execute_code` | `process_exec` |
| `start_process` | `process_exec` |
| `stop_process` | `process_exec` |
| `stream_output` | `process_exec` |
| `get_logs` | `memory_read` |
| `trace_execution` | `memory_read` |
| `measure_metrics` | `memory_read` |
| `debug_session` | `memory_read`, `process_exec` |
| `notify_user` | None |
| `request_approval` | None |
| `share_context` | `memory_read` |
| `comment` | `fs_write` (for files), `canvas_write` (for tiles) |
| `canvas_link_modules` | `canvas_write` |
| `canvas_search` | `canvas_read` |

### File Organization

```
src/tools/
├── analyze_code.rs       # NEW - Code analysis
├── collaboration.rs      # NEW - 4 collaboration tools
├── refactor_code.rs     # NEW - Refactoring tool
├── generate_tests.rs    # NEW - Test generation
├── review_code.rs      # NEW - Code review
├── find_bugs.rs         # NEW - Bug finding
├── execute_code.rs      # NEW - Code execution
├── evaluate_result.rs   # NEW - Result evaluation
├── monitoring.rs        # NEW - 3 monitoring tools
├── build.rs            # NEW - Build tool
└── ...existing tools...
```

---

## Testing Strategy

### Unit Tests
- All new tools should have unit tests
- Test edge cases (missing arguments, invalid inputs)
- Test error handling
- Verify capability enforcement

### Integration Tests
- Test tools in combination (e.g., `analyze_code` → `generate_tests` → `refactor_code`)
- Test capability interactions
- Test end-to-end workflows

### Benchmark Tests
- Measure performance of new tools
- Compare with existing tools
- Ensure tools meet performance targets:
  - `analyze_code` < 5s for files < 10KB
  - `generate_tests` < 10s for files < 50KB
  - `get_logs` retrieves 1000 entries in < 100ms
  - `trace_execution` renders trace tree in < 50ms

---

## Success Metrics

### Coverage Metrics (Post-Implementation)
- ✅ File System: 100% → 100%
- ✅ Command Execution: 25% → 100%
- ✅ Web & Network: 92% → 92%
- ✅ Canvas/Workspace: 60% → 80%
- ✅ Code Intelligence: 40% → 100% ✅
- ✅ Development Workflow: 75% → 100%
- ✅ Memory & Context: 100% → 100%
- ✅ Reasoning & Planning: 50% → 75% ✅
- ✅ Collaboration: 0% → 100% ✅
- ✅ Monitoring & Debugging: 30% → 75% ✅

**Overall Coverage**: 70% → 95% ✅ **+25%**

### Quality Metrics
- ✅ All tools follow existing patterns
- ✅ All tools have error handling
- ✅ All tools have documentation
- ✅ All tools have examples
- ✅ All tools are capability-gated

---

## Next Steps

1. ✅ **Review this document** - Ensure all tools are captured
2. ✅ **Prioritize by category** - High priority tools first
3. ✅ **Start with Phase 1** - Code intelligence and collaboration
4. ✅ **Create implementation plan** - Timeline with milestones
5. ✅ **Implement tools incrementally** - Test after each tool
6. ✅ **Document as you go** - Keep implementation guides updated
7. ✅ **Gather feedback** - Iterate based on usage patterns
8. ✅ **Proceed to Phase 2** - Based on Phase 1 results

---

**Last updated**: 2025-02-10
