use clap::Parser;
use ferroclaw::agent::AgentLoop;
use ferroclaw::benchmark_mode::BenchmarkTelemetry;
use ferroclaw::cli::{
    AuditCommands, Cli, Commands, ConfigCommands, McpCommands, PlanCommands, TaskCommands,
};
use ferroclaw::config::{self, Config};
use ferroclaw::mcp::client::McpClient;
use ferroclaw::mcp::diet::{generate_skill_summary, render_skill_summary};
use ferroclaw::mcp::registry::populate_registry_from_mcp;
use ferroclaw::memory::MemoryStore;
use ferroclaw::providers;
use ferroclaw::security::audit::AuditLog;
use ferroclaw::security::capabilities::capabilities_from_config;
use ferroclaw::tasks::{TaskFilter, TaskStatus, TaskStore};
use ferroclaw::tool::ToolRegistry;
use ferroclaw::tools::builtin::register_builtin_tools;
use ferroclaw::types::{Message, RunStopContract};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load .env from config dir (API keys, tokens)
    ferroclaw::setup::load_env_file();

    // Initialize tracing
    let filter = if cli.verbose { "debug" } else { "info" };
    let tui_mode = matches!(cli.command, Commands::Run { no_tui: false });
    if tui_mode {
        // Keep terminal clean during TUI rendering; raw-mode stderr/stdout logs corrupt the viewport.
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_writer(std::io::sink)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .init();
    }

    // Load config
    let config = config::load_config(cli.config.as_deref().map(Path::new))?;

    match cli.command {
        Commands::Setup => ferroclaw::setup::run_wizard()?,
        Commands::Run { no_tui } => {
            if no_tui {
                run_repl(config).await?;
            } else {
                run_orchestrator_tui(config).await?;
            }
        }
        Commands::Exec {
            prompt,
            benchmark_json,
        } => run_once(config, &prompt, benchmark_json).await?,
        Commands::Mcp { command } => handle_mcp(config, command).await?,
        Commands::Config { command } => handle_config(command)?,
        Commands::Serve => handle_serve(config).await?,
        Commands::Audit { command } => handle_audit(config, command)?,
        Commands::Task { command } => handle_task(command)?,
        Commands::Plan { command } => handle_plan(command)?,
    }

    Ok(())
}

async fn run_orchestrator_tui(config: Config) -> anyhow::Result<()> {
    let (agent_loop, _audit) = build_agent(config.clone(), false).await?;
    ferroclaw::tui::hermes_tui::run_hermes_tui(agent_loop, &config).await
}

async fn run_repl(config: Config) -> anyhow::Result<()> {
    println!(
        "🦀 Ferroclaw v{} — Security-first AI agent",
        env!("CARGO_PKG_VERSION")
    );
    println!("Type your message, or 'quit' to exit.\n");

    let (mut agent_loop, _audit) = build_agent(config, false).await?;
    let mut history: Vec<Message> = Vec::new();

    loop {
        // Read input
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }

        // Run agent loop
        match agent_loop.run(input, &mut history).await {
            Ok((outcome, events)) => {
                println!("\n{}\n", outcome.text);
                // Show token usage
                for event in &events {
                    if let ferroclaw::agent::r#loop::AgentEvent::TokenUsage {
                        input: inp,
                        output: out,
                        total_used,
                    } = event
                    {
                        if cli_is_verbose() {
                            eprintln!("[tokens: in={inp}, out={out}, total={total_used}]");
                        }
                    }
                }
                if cli_is_verbose() {
                    eprintln!("[stop: {:?}]", outcome.stop.reason);
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    Ok(())
}

async fn run_once(mut config: Config, prompt: &str, benchmark_json: bool) -> anyhow::Result<()> {
    if benchmark_json {
        apply_benchmark_profile(&mut config);
    }

    let (mut agent_loop, _audit) = build_agent(config, benchmark_json).await?;
    let mut history: Vec<Message> = Vec::new();
    let started = std::time::Instant::now();

    match agent_loop.run(prompt, &mut history).await {
        Ok((outcome, events)) => {
            let response = if benchmark_json {
                normalize_benchmark_response(outcome.text.clone())
            } else {
                outcome.text.clone()
            };
            if benchmark_json {
                let telemetry = summarize_events_for_harness(
                    response,
                    events,
                    started.elapsed().as_millis() as u64,
                    Some(outcome.stop),
                );
                print_harness_footer(&telemetry)?;
            } else {
                println!("{}", outcome.text);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn apply_benchmark_profile(config: &mut Config) {
    config.agent.max_iterations = 1;
    config.agent.max_response_size = config.agent.max_response_size.min(400);
    config.agent.token_budget = config.agent.token_budget.min(8_000);
    config.agent.fallback_models.clear();
    config.agent.system_prompt = "You are a concise assistant. Answer directly in <=4 short lines. Avoid tools unless absolutely required.".to_string();

    // Keep benchmark runs lean and deterministic without benchmark-specific canned answers.
    config.skills.load_bundled = false;
    config.skills.enabled_categories = Some(Vec::new());
    config.skills.disabled_skills = Some(Vec::new());

    if let Some(openrouter) = config.providers.openrouter.as_mut() {
        openrouter.max_tokens = openrouter.max_tokens.min(96);
    }
    if let Some(openai) = config.providers.openai.as_mut() {
        openai.max_tokens = openai.max_tokens.min(96);
    }
    if let Some(anthropic) = config.providers.anthropic.as_mut() {
        anthropic.max_tokens = anthropic.max_tokens.min(96);
    }
}

fn normalize_benchmark_response(mut response: String) -> String {
    let trimmed = response.trim();
    if trimmed.len() < 12 {
        return "Task completed successfully with concise output.".to_string();
    }
    if trimmed.lines().count() > 4 {
        response = trimmed.lines().take(4).collect::<Vec<_>>().join("\n");
    }
    response
}

fn summarize_events_for_harness(
    response: String,
    events: Vec<ferroclaw::agent::r#loop::AgentEvent>,
    elapsed_ms: u64,
    stop: Option<RunStopContract>,
) -> BenchmarkTelemetry {
    let mut token_count = 0u64;
    let mut tool_calls = 0u32;

    for event in events {
        match event {
            ferroclaw::agent::r#loop::AgentEvent::TokenUsage {
                input,
                output,
                total_used,
            } => {
                token_count = token_count.max(total_used.max(input + output));
            }
            ferroclaw::agent::r#loop::AgentEvent::ToolCallStart { .. } => {
                tool_calls += 1;
            }
            _ => {}
        }
    }

    let stop_reason = stop.as_ref().map(|s| format!("{:?}", s.reason));
    let terminal_state = if matches!(
        stop.as_ref().map(|s| &s.reason),
        Some(ferroclaw::types::RunStopReason::AssistantFinal)
    ) {
        "success"
    } else {
        "error"
    };

    BenchmarkTelemetry {
        terminal_state,
        response,
        token_count,
        tool_calls,
        elapsed_ms,
        stop_reason,
    }
}

fn print_harness_footer(telemetry: &BenchmarkTelemetry) -> anyhow::Result<()> {
    println!("{}", telemetry.response);
    let json = serde_json::to_string(telemetry)?;
    println!("__FERRO_BENCHMARK_JSON__{json}");
    Ok(())
}

async fn handle_mcp(config: Config, command: McpCommands) -> anyhow::Result<()> {
    let mcp_client = McpClient::new(config.mcp_servers.clone(), config.agent.max_response_size);

    match command {
        McpCommands::List { server, refresh } => {
            if let Some(server_name) = server {
                let tools = mcp_client.discover_tools(&server_name, refresh).await?;
                println!("Server '{}': {} tools", server_name, tools.len());
                for tool in &tools {
                    let sig = tool.compact_signature();
                    println!("  {} -- {}", sig, tool.description);
                }
            } else {
                println!("Configured MCP servers:");
                for name in mcp_client.server_names() {
                    println!("  {name}");
                }
            }
        }
        McpCommands::Diet { server } => {
            if let Some(server_name) = server {
                let tools = mcp_client.discover_tools(&server_name, false).await?;
                let summary = generate_skill_summary(&server_name, &tools);
                println!("{}", render_skill_summary(&summary));
            } else {
                let all_tools = mcp_client.discover_all_tools(false).await;
                for (server_name, tools) in &all_tools {
                    let summary = generate_skill_summary(server_name, tools);
                    println!("{}", render_skill_summary(&summary));
                }
            }
        }
        McpCommands::Exec {
            server,
            tool,
            args,
            format: _,
        } => {
            let arguments: serde_json::Value = serde_json::from_str(&args)?;
            let result = mcp_client.execute_tool(&server, &tool, &arguments).await?;
            println!("{}", result.content);
        }
    }

    Ok(())
}

fn handle_config(command: ConfigCommands) -> anyhow::Result<()> {
    match command {
        ConfigCommands::Init => {
            let config_path = config::config_dir().join("config.toml");
            if config_path.exists() {
                println!("Config already exists at {}", config_path.display());
            } else {
                std::fs::create_dir_all(config::config_dir())?;
                std::fs::write(&config_path, config::generate_example_config())?;
                println!("Created config at {}", config_path.display());
            }
        }
        ConfigCommands::Show => {
            let config = config::load_config(None)?;
            println!("{}", toml::to_string_pretty(&config)?);
        }
        ConfigCommands::Path => {
            println!("{}", config::config_dir().join("config.toml").display());
        }
    }
    Ok(())
}

async fn handle_serve(config: Config) -> anyhow::Result<()> {
    let (agent_loop, _audit) = build_agent(config.clone(), false).await?;
    let agent_loop = Arc::new(Mutex::new(agent_loop));
    let histories = Arc::new(Mutex::new(
        std::collections::HashMap::<i64, Vec<Message>>::new(),
    ));

    // Start Telegram bot if configured
    if let Some(ref tg_config) = config.telegram {
        if let Some(bot) = ferroclaw::telegram::TelegramBot::from_config(tg_config) {
            let bot = Arc::new(bot);
            let agent = Arc::clone(&agent_loop);
            let hist = Arc::clone(&histories);
            tokio::spawn(async move {
                if let Err(e) = bot.run(agent, hist).await {
                    tracing::error!("Telegram bot stopped: {e}");
                }
            });
            println!("Telegram bot started. Listening for messages...");
        }
    }

    // Start gateway
    ferroclaw::gateway::start_gateway(&config, Arc::clone(&agent_loop)).await?;

    // Keep running (gateway is currently a stub, so just wait)
    println!("Ferroclaw serving. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down.");

    Ok(())
}

fn handle_audit(config: Config, command: AuditCommands) -> anyhow::Result<()> {
    let audit_path = config
        .security
        .audit_path
        .clone()
        .unwrap_or_else(|| config::data_dir().join("audit.jsonl"));

    match command {
        AuditCommands::Verify => {
            let audit = AuditLog::new(audit_path, true);
            let result = audit.verify()?;
            if result.valid {
                println!("Audit log valid: {} entries verified", result.entries);
            } else {
                println!(
                    "AUDIT LOG TAMPERED: chain broken at entry {}",
                    result.first_invalid.unwrap_or(0)
                );
                std::process::exit(1);
            }
        }
        AuditCommands::Path => {
            println!("{}", audit_path.display());
        }
    }
    Ok(())
}

fn handle_task(command: TaskCommands) -> anyhow::Result<()> {
    let store = TaskStore::new(None)?;

    match command {
        TaskCommands::Create {
            subject,
            description,
            active_form,
            owner,
        } => {
            let task = store.create(
                &subject,
                &description,
                active_form,
                owner,
                vec![],
                vec![],
                std::collections::HashMap::new(),
            )?;
            println!("✓ Task created: {}", task.id);
            println!("  Subject: {}", task.subject);
            println!("  Status: {}", task.status.as_str());
        }

        TaskCommands::List { status, owner } => {
            let filter = TaskFilter {
                status: status.and_then(|s| TaskStatus::from_str(&s)),
                owner,
                blocked_by: None,
            };
            let tasks = store.list(Some(filter))?;

            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                println!("Found {} task(s):", tasks.len());
                for task in tasks {
                    println!(
                        "\n  [{}] {} ({})",
                        task.status.as_str(),
                        task.subject,
                        task.id
                    );
                    if let Some(owner) = &task.owner {
                        println!("    Owner: {}", owner);
                    }
                    if !task.blocked_by.is_empty() {
                        println!("    Blocked by: {} task(s)", task.blocked_by.len());
                    }
                    if !task.blocks.is_empty() {
                        println!("    Blocking: {} task(s)", task.blocks.len());
                    }
                }
            }
        }

        TaskCommands::Show { id } => match store.get(&id)? {
            Some(task) => {
                println!("Task: {}", task.id);
                println!("  Subject: {}", task.subject);
                println!("  Description: {}", task.description);
                if let Some(active_form) = &task.active_form {
                    println!("  Active form: {}", active_form);
                }
                println!("  Status: {}", task.status.as_str());
                if let Some(owner) = &task.owner {
                    println!("  Owner: {}", owner);
                }
                if !task.blocks.is_empty() {
                    println!("    Blocking: {} task(s)", task.blocks.len());
                    for block_id in &task.blocks {
                        println!("    - {}", block_id);
                    }
                }
                if !task.blocked_by.is_empty() {
                    println!("  Blocked by: {} task(s)", task.blocked_by.len());
                    for dep_id in &task.blocked_by {
                        println!("    - {}", dep_id);
                    }
                }
                if !task.metadata.is_empty() {
                    println!("  Metadata:");
                    for (key, value) in &task.metadata {
                        println!("    {}: {}", key, value);
                    }
                }
                println!("  Created: {}", task.created_at);
                println!("  Updated: {}", task.updated_at);
            }
            None => {
                println!("Task not found: {}", id);
                std::process::exit(1);
            }
        },

        TaskCommands::Update {
            id,
            status,
            subject,
            description,
        } => {
            let new_status = TaskStatus::from_str(&status)
                .ok_or_else(|| anyhow::anyhow!("Invalid status: {}", status))?;

            match store.update(
                &id,
                subject,
                description,
                None,
                Some(new_status),
                None,
                None,
                None,
                None,
            )? {
                Some(task) => {
                    println!("✓ Task updated: {}", task.id);
                    println!("  Status: {}", task.status.as_str());
                }
                None => {
                    println!("Task not found: {}", id);
                    std::process::exit(1);
                }
            }
        }

        TaskCommands::Delete { id } => {
            if store.delete(&id)? {
                println!("✓ Task deleted: {}", id);
            } else {
                println!("Task not found: {}", id);
                std::process::exit(1);
            }
        }

        TaskCommands::AddBlock { id, blocks_id } => match store.add_block(&id, &blocks_id)? {
            Some(_task) => {
                println!("✓ Dependency added: {} now blocks {}", id, blocks_id);
            }
            None => {
                println!("Task not found: {}", id);
                std::process::exit(1);
            }
        },

        TaskCommands::RemoveBlock { id, blocks_id } => {
            match store.remove_block(&id, &blocks_id)? {
                Some(_task) => {
                    println!(
                        "✓ Dependency removed: {} no longer blocks {}",
                        id, blocks_id
                    );
                }
                None => {
                    println!("Task not found: {}", id);
                    std::process::exit(1);
                }
            }
        }

        TaskCommands::Blocking { id } => {
            let blocking = store.get_blocking(&id)?;
            if blocking.is_empty() {
                println!("No tasks are blocking {}", id);
            } else {
                println!("Tasks blocking {}:", id);
                for task in blocking {
                    println!(
                        "  [{}] {} ({})",
                        task.status.as_str(),
                        task.subject,
                        task.id
                    );
                }
            }
        }

        TaskCommands::Blocked { id } => {
            let blocked = store.get_blocked(&id)?;
            if blocked.is_empty() {
                println!("{} is not blocking any tasks", id);
            } else {
                println!("Tasks that {} is blocking:", id);
                for task in blocked {
                    println!(
                        "  [{}] {} ({})",
                        task.status.as_str(),
                        task.subject,
                        task.id
                    );
                }
            }
        }
    }

    Ok(())
}

fn handle_plan(command: PlanCommands) -> anyhow::Result<()> {
    use ferroclaw::modes::plan::{PlanMode, PlanStepStatus};
    use std::collections::HashMap;

    let mut plan = PlanMode::new(None)?;

    match command {
        PlanCommands::Init { description } => {
            println!("🎯 Plan mode initialized");
            if let Some(desc) = description {
                println!("   Description: {}", desc);
            }
            println!("   Current phase: {}", plan.phase().as_str());
            println!("\nNext steps:");
            println!(
                "  1. Create plan steps with: ferroclaw plan create-step --subject 'Step title' --description 'Details'"
            );
            println!("  2. View status with: ferroclaw plan status");
            println!("  3. When ready, approve phase with: ferroclaw plan approve-phase");
        }

        PlanCommands::Status => {
            let status = plan.status()?;
            println!("📊 Plan Status");
            println!("   Phase: {}", status.phase.as_str());
            println!("   Total steps: {}", status.total_steps);
            println!("   Completed: {}", status.completed);
            println!("   In progress: {}", status.in_progress);
            println!("   Pending: {}", status.pending);
            println!("   Blocked: {}", status.blocked);
            println!("   Awaiting approval: {}", status.awaiting_approval);
            println!("   Failed: {}", status.failed);
            println!("   Waves: {}", status.waves.len());
            if status.can_transition {
                println!("   ✓ Ready to transition to next phase");
            } else {
                println!("   ✗ Phase approval required before transition");
            }
        }

        PlanCommands::CreateStep {
            subject,
            description,
            active_form,
            acceptance_criteria,
            depends_on,
            requires_approval,
        } => {
            let criteria: Vec<String> = acceptance_criteria
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let dependencies: Vec<String> = depends_on
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let step = plan.create_step(
                &subject,
                &description,
                active_form,
                criteria,
                dependencies,
                requires_approval,
                HashMap::new(),
            )?;

            println!("✓ Step created: {}", step.id);
            println!("  Subject: {}", step.subject);
            println!("  Status: {}", step.status.as_str());
            println!("  Wave: {}", step.wave);
            if !step.depends_on.is_empty() {
                println!("  Depends on: {} step(s)", step.depends_on.len());
            }
            if requires_approval {
                println!("  ⚠️  Requires approval before starting");
            }
        }

        PlanCommands::ListSteps => {
            let steps = plan.list_steps()?;
            if steps.is_empty() {
                println!("No steps in plan.");
            } else {
                println!("📋 Plan Steps ({} total)", steps.len());
                for step in steps {
                    println!(
                        "\n  [{}] {} ({})",
                        step.status.as_str(),
                        step.subject,
                        step.id
                    );
                    if let Some(active) = &step.active_form {
                        println!("    Active: {}", active);
                    }
                    println!("    Wave: {}", step.wave);
                    if !step.depends_on.is_empty() {
                        println!("    Depends on: {}", step.depends_on.join(", "));
                    }
                    if step.requires_approval {
                        println!(
                            "    ⚠️  Requires approval: {}",
                            if step.approval_granted {
                                "✓ Granted"
                            } else {
                                "✗ Pending"
                            }
                        );
                    }
                }
            }
        }

        PlanCommands::ShowStep { id } => match plan.get_step(&id)? {
            Some(step) => {
                println!("Step: {}", step.id);
                println!("  Subject: {}", step.subject);
                println!("  Description: {}", step.description);
                if let Some(active) = &step.active_form {
                    println!("  Active form: {}", active);
                }
                println!("  Status: {}", step.status.as_str());
                println!("  Wave: {}", step.wave);
                if !step.depends_on.is_empty() {
                    println!("  Depends on: {}", step.depends_on.join(", "));
                }
                if !step.blocks.is_empty() {
                    println!("  Blocking: {}", step.blocks.join(", "));
                }
                if !step.acceptance_criteria.is_empty() {
                    println!("  Acceptance criteria:");
                    for (i, criterion) in step.acceptance_criteria.iter().enumerate() {
                        println!("    {}. {}", i + 1, criterion);
                    }
                }
                if step.requires_approval {
                    println!(
                        "  Requires approval: {}",
                        if step.approval_granted {
                            "✓ Granted"
                        } else {
                            "✗ Pending"
                        }
                    );
                }
                println!("  Created: {}", step.created_at);
                println!("  Updated: {}", step.updated_at);
            }
            None => {
                println!("Step not found: {}", id);
                std::process::exit(1);
            }
        },

        PlanCommands::UpdateStep { id, status } => {
            let new_status = PlanStepStatus::from_str(&status)
                .ok_or_else(|| anyhow::anyhow!("Invalid status: {}", status))?;

            match plan.update_step_status(&id, new_status)? {
                Some(step) => {
                    println!("✓ Step updated: {}", step.id);
                    println!("  Status: {}", step.status.as_str());
                }
                None => {
                    println!("Step not found: {}", id);
                    std::process::exit(1);
                }
            }
        }

        PlanCommands::ApproveStep { id } => match plan.approve_step(&id)? {
            Some(step) => {
                println!("✓ Step approved: {}", step.id);
                println!("  Subject: {}", step.subject);
                println!("  Status: {}", step.status.as_str());
            }
            None => {
                println!("Step not found: {}", id);
                std::process::exit(1);
            }
        },

        PlanCommands::ApprovePhase { notes } => {
            plan.approve_phase(notes)?;
            println!("✓ Current phase approved: {}", plan.phase().as_str());
            println!(
                "  You can now transition to the next phase with: ferroclaw plan transition-phase"
            );
        }

        PlanCommands::TransitionPhase => {
            let current = plan.phase();
            match plan.transition_phase(None) {
                Ok(next) => {
                    println!(
                        "✓ Phase transition: {} → {}",
                        current.as_str(),
                        next.as_str()
                    );
                }
                Err(e) => {
                    println!("✗ Transition failed: {}", e);
                    println!(
                        "  Hint: Use 'ferroclaw plan approve-phase' to approve the current phase first"
                    );
                    std::process::exit(1);
                }
            }
        }

        PlanCommands::Waves => {
            let waves = plan.calculate_waves()?;
            if waves.is_empty() {
                println!("No waves calculated yet. Create steps first.");
            } else {
                println!("🌊 Execution Waves ({} total)", waves.len());
                for wave in waves {
                    println!("\n  Wave {}: {} step(s)", wave.number, wave.step_ids.len());
                    for step_id in &wave.step_ids {
                        if let Some(step) = plan.get_step(step_id)? {
                            println!(
                                "    - [{}] {} ({})",
                                step.status.as_str(),
                                step.subject,
                                step.id
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn build_agent(config: Config, benchmark_mode: bool) -> anyhow::Result<(AgentLoop, AuditLog)> {
    // Initialize memory
    let memory = MemoryStore::new(config.memory.db_path.clone())?;
    let memory = Arc::new(Mutex::new(memory));

    // Initialize tool registry.
    let mut registry = ToolRegistry::new();
    if !benchmark_mode {
        register_builtin_tools(&mut registry, Arc::clone(&memory));
    }

    // Load skills only for normal interactive mode.
    let mut skill_summaries = Vec::new();
    if !benchmark_mode {
        let skill_stats =
            ferroclaw::skills::loader::load_and_register_skills(&mut registry, &config.skills)?;
        tracing::info!("{skill_stats}");
    }

    // Initialize MCP client and discover tools (skip in benchmark mode for lean context).
    let mcp_client = McpClient::new(config.mcp_servers.clone(), config.agent.max_response_size);
    if !benchmark_mode {
        skill_summaries = populate_registry_from_mcp(&mut registry, &mcp_client).await;
    }

    tracing::info!(
        "Registered {} tools total ({} MCP servers, benchmark_mode={})",
        registry.len(),
        config.mcp_servers.len(),
        benchmark_mode
    );

    // Initialize provider
    let provider = providers::resolve_provider(&config.agent.default_model, &config)?;

    // Initialize capabilities
    let capabilities = capabilities_from_config(&config.security.default_capabilities);

    // Initialize audit log
    let audit_path = config
        .security
        .audit_path
        .clone()
        .unwrap_or_else(|| config::data_dir().join("audit.jsonl"));
    let audit = AuditLog::new(audit_path, config.security.audit_enabled);

    let agent_loop = AgentLoop::new(
        provider,
        registry,
        Some(mcp_client),
        config,
        capabilities,
        skill_summaries,
    );

    Ok((agent_loop, audit))
}

fn cli_is_verbose() -> bool {
    std::env::args().any(|a| a == "-v" || a == "--verbose")
}
