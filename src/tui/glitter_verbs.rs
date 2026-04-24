//! Human-readable "glitter" labels for orchestrator phases (Claude Code–style short verbs).
//! Ported from Agent Canvas orchestrator implementation.

use std::time::Instant;

/// Glitter verb for the prepare phase.
pub fn glitter_verb_for_prepare() -> &'static str {
    "Preparing session…"
}

/// LLM pending verbs - cycling through different thoughtful messages.
const LLM_PENDING_VERBS: &[&str] = &[
    "Contemplating…",
    "Kerplunking through context…",
    "Cannonballing into the prompt…",
    "Synthesizing…",
    "Pondering…",
    "Interrogating the vibes…",
    "Triangulating an answer…",
    "Mulling with intent…",
    "Orbiting the problem…",
    "Cross-referencing the universe…",
    "Concocting a response…",
    "Massaging the latent space…",
    "Scheming productively…",
    "Doing a little epistemology…",
    "Nibbling on possibilities…",
    "Calibrating brilliance…",
    "Juggling hypotheses…",
    "Whispering to the tensors…",
    "Assembling cleverness…",
    "Reasoning with panache…",
];

/// Time bucket for cycling through verbs (milliseconds).
const LLM_PENDING_BUCKET_MS: u64 = 2200;

/// Get an LLM pending verb based on elapsed time and iteration.
pub fn glitter_verb_for_llm_pending(elapsed_ms: u128, iteration: u32) -> String {
    let bucket = elapsed_ms.saturating_div(LLM_PENDING_BUCKET_MS as u128) as usize;
    let round_offset = iteration.saturating_sub(1) * 5;
    // Deterministic pseudo-random walk through the list
    let idx = (bucket * 7 + 3 + round_offset as usize) % LLM_PENDING_VERBS.len();
    let verb = LLM_PENDING_VERBS[idx];

    let elapsed_secs = elapsed_ms.saturating_div(1000);
    if elapsed_secs == 0 {
        verb.to_string()
    } else if elapsed_secs < 45 {
        format!("{verb} · {elapsed_secs}s")
    } else {
        format!("{verb} · {elapsed_secs}s · tool rounds can be slow")
    }
}

/// Tool-specific glitter verbs.
fn glitter_verb_for_tool(name: &str) -> &'static str {
    match name {
        "read_file" => "Reading…",
        "write_file" => "Writing…",
        "delete_file" => "Deleting…",
        "list_directory" => "Listing…",
        "open_workspace" => "Opening workspace…",
        "canvas_list_modules" => "Scanning canvas…",
        "canvas_create_tile" => "Adding tiles…",
        "canvas_update_tile" => "Updating canvas…",
        _ => "Running tool…",
    }
}

/// Get glitter verb for multiple tools.
pub fn glitter_verb_for_tools(names: &[String]) -> String {
    let unique: Vec<&String> = names.iter().filter(|s| !s.is_empty()).collect();

    if unique.is_empty() {
        return "Working…".to_string();
    }

    if unique.len() == 1 {
        let name = unique[0];
        let verb = glitter_verb_for_tool(name);
        return format!("{verb}");
    }

    // Check if all tools are the same
    let all_same = unique.iter().all(|x| **x == **unique[0]);
    if all_same {
        let base = glitter_verb_for_tool(unique[0]);
        let stripped = base.trim_end_matches('…');
        return format!("{stripped} (×{})…", unique.len());
    }

    format!("Running {} tools…", unique.len())
}

/// Get glitter verb for LLM round.
pub fn glitter_verb_for_llm(iteration: u32) -> &'static str {
    if iteration <= 1 {
        "Round 1 — calling model…"
    } else {
        "Thinking…"
    }
}

/// Calculate elapsed milliseconds since start.
pub fn elapsed_ms_since(start: Option<Instant>) -> u128 {
    match start {
        Some(s) => s.elapsed().as_millis(),
        None => 0,
    }
}

/// Get the appropriate glitter verb based on current state.
pub fn get_glitter_verb(
    is_running: bool,
    iteration: u32,
    active_tools: &[String],
    run_started_at: Option<Instant>,
) -> String {
    if !is_running {
        "ready".to_string()
    } else if !active_tools.is_empty() {
        glitter_verb_for_tools(active_tools)
    } else {
        let elapsed = elapsed_ms_since(run_started_at);
        glitter_verb_for_llm_pending(elapsed, iteration)
    }
}
