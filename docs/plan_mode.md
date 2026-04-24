# PlanMode - Structured Multi-Phase Planning

## Overview

PlanMode provides a structured approach to complex, multi-step tasks through a four-phase system: Research, Planning, Implementation, and Verification. It integrates with TaskSystem for persistent storage and dependency tracking.

**Key Features:**
- Four-phase workflow (Research → Planning → Implementation → Verification)
- Dependency-based wave execution
- Approval gates for phase transitions
- Acceptance criteria per step
- Persistent storage via TaskSystem

## Phases

### Phase 1: Research

**Goal:** Gather information and understand requirements

**Activities:**
- Explore codebase structure
- Research existing solutions
- Identify dependencies
- Understand constraints

**Output:**
- Requirements document
- Architecture notes
- Risk assessment

**Example:**
```rust
use ferroclaw::modes::plan::{PlanMode, PlanPhase};

let mut plan = PlanMode::new(None)?;
assert_eq!(plan.current_phase(), PlanPhase::Research);

// Complete research
plan.advance_phase()?; // → Planning
```

### Phase 2: Planning

**Goal:** Create detailed steps with dependencies

**Activities:**
- Break down work into steps
- Define acceptance criteria
- Map dependencies
- Estimate effort

**Output:**
- PlanStep list with dependencies
- Wave execution plan
- Resource allocation

**Example:**
```rust
plan.add_step(
    "Design database schema",
    "Create ERD and table definitions",
    Some("Designing database schema".into()),
    vec!["Understand requirements".into()],
    vec![],
    vec!["Schema documented".into(), "Tables defined".into()],
    false,
)?;

plan.add_step(
    "Implement user table",
    "Create users table with auth fields",
    Some("Implementing users table".into()),
    vec!["Design database schema".into()],
    vec![],
    vec!["Table created".into(), "Indexes defined".into()],
    false,
)?;
```

### Phase 3: Implementation

**Goal:** Execute steps in dependency-based waves

**Activities:**
- Execute steps in order
- Update step status
- Handle blockers
- Track progress

**Output:**
- Completed implementation
- Status updates
- Test results

**Example:**
```rust
plan.advance_phase()?; // → Implementation

// Get current wave (steps with no dependencies)
let wave = plan.current_wave()?;
for step_id in &wave.step_ids {
    let step = plan.get_step(step_id)?;
    println!("Executing: {}", step.subject);
    // Execute step...
    plan.complete_step(step_id)?;
}
```

### Phase 4: Verification

**Goal:** Validate outcomes against acceptance criteria

**Activities:**
- Verify acceptance criteria
- Run tests
- Check quality metrics
- Document results

**Output:**
- Verification report
- Test coverage
- Quality metrics

**Example:**
```rust
plan.advance_phase()?; // → Verification

let step = plan.get_step(&step_id)?;
for criterion in &step.acceptance_criteria {
    // Verify criterion
    if verify_criterion(criterion)? {
        println!("✓ {}", criterion);
    } else {
        println!("✗ {}", criterion);
    }
}
```

## PlanStep Structure

Every step contains:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier (matches Task ID) |
| `subject` | string | Brief title |
| `description` | string | Detailed description |
| `active_form` | string | Present tense for progress display |
| `acceptance_criteria` | array[string] | Verification checklist |
| `depends_on` | array[string] | Step IDs this step depends on |
| `blocks` | array[string] | Step IDs that depend on this step |
| `status` | enum | Current status |
| `wave` | number | Wave number (0 = can start immediately) |
| `requires_approval` | boolean | Whether approval needed before execution |
| `approval_granted` | boolean | Whether approval was granted |
| `metadata` | object | Additional data |
| `created_at` | timestamp | Creation time |
| `updated_at` | timestamp | Last update time |

### Status Values

| Status | Description |
|--------|-------------|
| `Pending` | Not yet started |
| `InProgress` | Currently being worked on |
| `Completed` | Finished successfully |
| `Blocked` | Waiting for dependencies |
| `AwaitingApproval` | Waiting for approval gate |
| `Failed` | Failed and needs attention |

## Wave Execution

### What are Waves?

Waves organize steps into parallelizable groups based on dependencies:

```
Wave 0: Steps with no dependencies (can start immediately)
Wave 1: Steps whose dependencies are in Wave 0
Wave 2: Steps whose dependencies are in Waves 0-1
...
```

### Example Wave Structure

```
Step 1: "Design DB" (no dependencies)
         ↓ Wave 0

Step 2: "Implement users" (depends on Step 1)
Step 3: "Implement posts" (depends on Step 1)
         ↓ Wave 1

Step 4: "Write tests" (depends on Steps 2, 3)
         ↓ Wave 2
```

**Execution:**
```rust
// Wave 0
plan.execute_wave(0)?; // Executes Step 1

// Wave 1 (parallel)
plan.execute_wave(1)?; // Executes Steps 2 and 3 in parallel

// Wave 2
plan.execute_wave(2)?; // Executes Step 4
```

### Calculating Waves

```rust
let waves = plan.calculate_waves()?;

for wave in &waves {
    println!("Wave {}:", wave.number);
    for step_id in &wave.step_ids {
        let step = plan.get_step(step_id)?;
        println!("  - {}", step.subject);
    }
}
```

## Approval Workflow

### Setting Up Approval Gates

```rust
// Create step requiring approval
plan.add_step(
    "Deploy to production",
    "Deploy application to production servers",
    Some("Deploying to production".into()),
    vec!["All tests pass".into()],
    vec![],
    vec!["Deployment successful".into(), "Monitoring active".into()],
    true, // requires_approval = true
)?;

// Check if approval needed
let step = plan.get_step(&step_id)?;
if step.requires_approval && !step.approval_granted {
    println!("Awaiting approval before executing");
}
```

### Granting Approval

```rust
// Grant approval
plan.grant_approval(&step_id)?;

// Now step can be executed
let step = plan.get_step(&step_id)?;
assert!(step.approval_granted);

plan.execute_step(&step_id)?;
```

## Usage Examples

### Example 1: Complete Workflow

```rust
use ferroclaw::modes::plan::{PlanMode, PlanPhase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut plan = PlanMode::new(None)?;

    // PHASE 1: Research
    assert_eq!(plan.current_phase(), PlanPhase::Research);

    // Do research...
    println!("Researching requirements...");

    // Advance to planning
    plan.advance_phase()?;

    // PHASE 2: Planning
    assert_eq!(plan.current_phase(), PlanPhase::Planning);

    // Create steps
    plan.add_step(
        "Design API",
        "Design REST API endpoints",
        Some("Designing API".into()),
        vec![],
        vec![],
        vec!["Endpoints documented".into(), "Schema defined".into()],
        false,
    )?;

    plan.add_step(
        "Implement API",
        "Implement REST endpoints",
        Some("Implementing API".into()),
        vec!["Design API".into()],
        vec![],
        vec!["All endpoints working".into(), "Tests passing".into()],
        false,
    )?;

    // Calculate waves
    let waves = plan.calculate_waves()?;
    println!("Plan requires {} waves", waves.len());

    // Advance to implementation
    plan.advance_phase()?;

    // PHASE 3: Implementation
    assert_eq!(plan.current_phase(), PlanPhase::Implementation);

    // Execute wave 0
    let wave0 = plan.current_wave()?;
    for step_id in &wave0.step_ids {
        plan.start_step(step_id)?;
        // Execute step...
        plan.complete_step(step_id)?;
    }

    // Execute wave 1
    let wave1 = plan.next_wave()?;
    for step_id in &wave1.step_ids {
        plan.start_step(step_id)?;
        // Execute step...
        plan.complete_step(step_id)?;
    }

    // Advance to verification
    plan.advance_phase()?;

    // PHASE 4: Verification
    assert_eq!(plan.current_phase(), PlanPhase::Verification);

    // Verify all steps
    let steps = plan.all_steps()?;
    for step in steps {
        for criterion in &step.acceptance_criteria {
            println!("Verifying: {} - {}", step.subject, criterion);
            // Verify criterion...
        }
    }

    Ok(())
}
```

### Example 2: Dependency Management

```rust
// Create complex dependency graph
plan.add_step("Step 1", "...", None, vec![], vec![], vec![], false)?;
plan.add_step("Step 2", "...", None, vec!["Step 1".into()], vec![], vec![], false)?;
plan.add_step("Step 3", "...", None, vec!["Step 1".into()], vec![], vec![], false)?;
plan.add_step("Step 4", "...", None, vec!["Step 2".into(), "Step 3".into()], vec![], vec![], false)?;

// Waves:
// Wave 0: [Step 1]
// Wave 1: [Step 2, Step 3] (parallel)
// Wave 2: [Step 4]
```

### Example 3: Status Tracking

```rust
// Start a step
plan.start_step(&step_id)?;
let step = plan.get_step(&step_id)?;
assert_eq!(step.status, PlanStepStatus::InProgress);

// Complete a step
plan.complete_step(&step_id)?;
let step = plan.get_step(&step_id)?;
assert_eq!(step.status, PlanStepStatus::Completed);

// Fail a step
plan.fail_step(&step_id, "Database connection failed")?;
let step = plan.get_step(&step_id)?;
assert_eq!(step.status, PlanStepStatus::Failed);
```

## Integration with TaskSystem

PlanMode uses TaskSystem for persistent storage:

```rust
use ferroclaw::tasks::TaskStore;

// PlanMode creates TaskStore internally
let plan = PlanMode::new(None)?;

// PlanSteps are backed by Tasks
let step = plan.get_step(&step_id)?;
let task = plan.store().get(&step_id)?;

// Step.id == Task.id
assert_eq!(step.id, task.id);
```

### Benefits

- **Persistent storage**: Plans survive restarts
- **Dependency tracking**: Automatic cycle detection
- **Status management**: Centralized status tracking
- **Query capabilities**: Filter, search, sort

## Best Practices

### 1. Break Down Large Steps

```rust
// Bad
plan.add_step(
    "Build entire application",
    "Implement everything",
    None,
    vec![],
    vec![],
    vec!["Everything done".into()],
    false,
)?;

// Good
plan.add_step("Design architecture", "...", None, vec![], vec![], vec![], false)?;
plan.add_step("Implement core", "...", None, vec!["Design architecture".into()], vec![], vec![], false)?;
plan.add_step("Add tests", "...", None, vec!["Implement core".into()], vec![], vec![], false)?;
```

### 2. Define Clear Acceptance Criteria

```rust
// Bad
vec!["Done".into()]

// Good
vec![
    "Unit tests pass (80%+ coverage)".into(),
    "Integration tests pass".into(),
    "Documentation complete".into(),
    "Code reviewed".into(),
]
```

### 3. Use Approval Gates for Critical Steps

```rust
plan.add_step(
    "Deploy to production",
    "Deploy to prod servers",
    None,
    vec!["All tests pass".into()],
    vec![],
    vec!["Deployment successful".into()],
    true, // Requires approval
)?;
```

### 4. Check Dependencies Before Execution

```rust
let step = plan.get_step(&step_id)?;
if !step.depends_on.is_empty() {
    // Check if dependencies are complete
    for dep_id in &step.depends_on {
        let dep = plan.get_step(dep_id)?;
        if dep.status != PlanStepStatus::Completed {
            println!("Dependency not complete: {}", dep.subject);
        }
    }
}
```

### 5. Update Status Regularly

```rust
// Start step
plan.start_step(&step_id)?;

// If error occurs
plan.fail_step(&step_id, &error_message)?;

// On completion
plan.complete_step(&step_id)?;
```

## Troubleshooting

### Issue: Cannot advance phase

**Cause**: Previous phase not complete

**Solution**:
```rust
// Check phase requirements
match plan.current_phase() {
    PlanPhase::Research => {
        // Complete research tasks
    }
    PlanPhase::Planning => {
        // Ensure steps are defined
        let steps = plan.all_steps()?;
        if steps.is_empty() {
            return Err("No steps defined".into());
        }
    }
    PlanPhase::Implementation => {
        // Ensure all steps are completed
        for step in plan.all_steps()? {
            if step.status != PlanStepStatus::Completed {
                return Err(format!("Step not complete: {}", step.subject).into());
            }
        }
    }
    PlanPhase::Verification => {
        // Terminal phase
    }
}
```

### Issue: Step won't execute (blocked)

**Cause**: Dependencies not complete

**Solution**:
```rust
let step = plan.get_step(&step_id)?;
if step.status == PlanStepStatus::Blocked {
    println!("Step is blocked by:");
    for dep_id in &step.depends_on {
        let dep = plan.get_step(dep_id)?;
        println!("  - {} ({})", dep.subject, dep.status);
    }
}
```

### Issue: Cycle detected error

**Cause**: Circular dependency in steps

**Solution**:
```rust
// Visualize dependencies
let steps = plan.all_steps()?;
for step in &steps {
    if !step.depends_on.is_empty() {
        println!("{} depends on:", step.subject);
        for dep_id in &step.depends_on {
            let dep = plan.get_step(dep_id)?;
            println!("  - {}", dep.subject);
        }
    }
}

// Fix circular dependencies
// Bad: A → B → A
// Good: A → C, B → C (both depend on C)
```

## See Also

- **TaskSystem**: Persistent task storage
- **MemdirSystem**: File-based memory for plans
- **AgentTool**: Subagent spawning for step execution
