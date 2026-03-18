//! Benchmarks for the SQLite + FTS5 memory store.
//!
//! Measures:
//! - Insert throughput
//! - FTS5 search latency
//! - Conversation persistence throughput
//! - List all with various sizes

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ferroclaw::memory::MemoryStore;

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_insert");

    for count in [10, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("sequential", count),
            &count,
            |b, &count| {
                b.iter_with_setup(
                    || MemoryStore::in_memory().unwrap(),
                    |store| {
                        for i in 0..count {
                            store
                                .insert(
                                    &format!("key_{i}"),
                                    &format!("Value content for item number {i} with some padding text"),
                                )
                                .unwrap();
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_search");

    for count in [50, 200, 1000] {
        group.bench_with_input(
            BenchmarkId::new("fts5_query", count),
            &count,
            |b, &count| {
                b.iter_with_setup(
                    || {
                        let store = MemoryStore::in_memory().unwrap();
                        for i in 0..count {
                            store
                                .insert(
                                    &format!("item_{i}"),
                                    &format!(
                                        "This is item {i} about {} technology and {} framework",
                                        if i % 3 == 0 { "Rust" } else if i % 3 == 1 { "Python" } else { "JavaScript" },
                                        if i % 2 == 0 { "agent" } else { "compiler" }
                                    ),
                                )
                                .unwrap();
                        }
                        store
                    },
                    |store| {
                        let _ = store.search(black_box("Rust agent"), 10).unwrap();
                    },
                );
            },
        );
    }

    group.finish();
}

fn bench_conversation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_conversation");

    for turns in [10, 50, 200] {
        group.bench_with_input(
            BenchmarkId::new("save_turns", turns),
            &turns,
            |b, &turns| {
                b.iter_with_setup(
                    || MemoryStore::in_memory().unwrap(),
                    |store| {
                        for i in 0..turns {
                            let role = if i % 2 == 0 { "user" } else { "assistant" };
                            store
                                .save_conversation("sess_bench", role, &format!("Message {i} with content"))
                                .unwrap();
                        }
                    },
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("retrieve_turns", turns),
            &turns,
            |b, &turns| {
                b.iter_with_setup(
                    || {
                        let store = MemoryStore::in_memory().unwrap();
                        for i in 0..turns {
                            let role = if i % 2 == 0 { "user" } else { "assistant" };
                            store
                                .save_conversation("sess_bench", role, &format!("Message {i}"))
                                .unwrap();
                        }
                        store
                    },
                    |store| {
                        let _ = store.get_conversation(black_box("sess_bench")).unwrap();
                    },
                );
            },
        );
    }

    group.finish();
}

fn bench_list_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_list_all");

    for count in [50, 200, 1000] {
        group.bench_with_input(
            BenchmarkId::new("list", count),
            &count,
            |b, &count| {
                b.iter_with_setup(
                    || {
                        let store = MemoryStore::in_memory().unwrap();
                        for i in 0..count {
                            store.insert(&format!("k_{i}"), &format!("v_{i}")).unwrap();
                        }
                        store
                    },
                    |store| {
                        let _ = store.list_all().unwrap();
                    },
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_insert, bench_search, bench_conversation, bench_list_all);
criterion_main!(benches);
