//! Benchmarks for DietMCP compression performance.
//!
//! Measures:
//! - Schema compression time for various tool counts
//! - Compact signature generation
//! - Response formatting (summary, minified, CSV)
//! - Auto-redirect for large responses

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ferroclaw::mcp::diet::{
    format_response, generate_skill_summary, render_skill_summary, DietFormat,
};
use ferroclaw::types::ToolDefinition;
use serde_json::json;

fn make_tools(count: usize) -> Vec<ToolDefinition> {
    (0..count)
        .map(|i| ToolDefinition {
            name: format!("tool_{i}"),
            description: format!(
                "This is tool number {i} which performs an important operation on files and data"
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "The file path to operate on"},
                    "content": {"type": "string", "description": "Content to process"},
                    "options": {
                        "type": "object",
                        "properties": {
                            "recursive": {"type": "boolean"},
                            "depth": {"type": "integer"},
                            "pattern": {"type": "string"}
                        }
                    },
                    "format": {
                        "type": "string",
                        "enum": ["json", "text", "csv", "yaml"]
                    }
                },
                "required": ["path"]
            }),
            server_name: Some("benchmark".into()),
        })
        .collect()
}

fn bench_skill_summary_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("diet_skill_summary");

    for count in [5, 10, 25, 50, 100] {
        let tools = make_tools(count);
        group.bench_with_input(
            BenchmarkId::new("generate", count),
            &tools,
            |b, tools| {
                b.iter(|| generate_skill_summary(black_box("bench_server"), black_box(tools)))
            },
        );
    }

    group.finish();
}

fn bench_render_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("diet_render");

    for count in [10, 50, 100] {
        let tools = make_tools(count);
        let summary = generate_skill_summary("bench_server", &tools);
        group.bench_with_input(
            BenchmarkId::new("render", count),
            &summary,
            |b, summary| b.iter(|| render_skill_summary(black_box(summary))),
        );
    }

    group.finish();
}

fn bench_compact_signature(c: &mut Criterion) {
    let tool = ToolDefinition {
        name: "complex_tool".into(),
        description: "A complex tool with many parameters".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer"},
                "offset": {"type": "integer"},
                "filters": {"type": "array", "items": {"type": "string"}},
                "sort_by": {"type": "string", "enum": ["name", "date", "size", "relevance"]},
                "recursive": {"type": "boolean"},
                "format": {"type": "string"}
            },
            "required": ["query"]
        }),
        server_name: None,
    };

    c.bench_function("compact_signature", |b| {
        b.iter(|| black_box(&tool).compact_signature())
    });
}

fn bench_format_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("diet_format");

    // JSON response of various sizes
    for size in [1_000, 10_000, 50_000] {
        let data: Vec<serde_json::Value> = (0..size / 50)
            .map(|i| {
                json!({
                    "name": format!("item_{i}"),
                    "value": i,
                    "active": i % 2 == 0
                })
            })
            .collect();
        let content = serde_json::to_string(&data).unwrap();

        group.bench_with_input(
            BenchmarkId::new("summary", size),
            &content,
            |b, content| {
                b.iter(|| format_response(black_box(content), DietFormat::Summary, 50_000))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("minified", size),
            &content,
            |b, content| {
                b.iter(|| format_response(black_box(content), DietFormat::Minified, 50_000))
            },
        );

        group.bench_with_input(BenchmarkId::new("csv", size), &content, |b, content| {
            b.iter(|| format_response(black_box(content), DietFormat::Csv, 50_000))
        });
    }

    group.finish();
}

fn bench_compression_ratio(c: &mut Criterion) {
    let tools = make_tools(50);
    let raw = serde_json::to_string(&tools).unwrap();

    c.bench_function("compression_ratio_50_tools", |b| {
        b.iter(|| {
            let summary = generate_skill_summary("server", black_box(&tools));
            let rendered = render_skill_summary(&summary);
            let ratio = 1.0 - (rendered.len() as f64 / raw.len() as f64);
            black_box(ratio)
        })
    });
}

criterion_group!(
    benches,
    bench_skill_summary_generation,
    bench_render_summary,
    bench_compact_signature,
    bench_format_response,
    bench_compression_ratio,
);
criterion_main!(benches);
