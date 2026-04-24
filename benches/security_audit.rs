//! Benchmarks for the security subsystem.
//!
//! Measures:
//! - Capability checking performance
//! - Audit log write throughput
//! - Audit log verification time
//! - Hash chain computation

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ferroclaw::security::audit::AuditLog;
use ferroclaw::types::{Capability, CapabilitySet};
use tempfile::TempDir;

fn bench_capability_check(c: &mut Criterion) {
    let caps = CapabilitySet::new([
        Capability::FsRead,
        Capability::NetOutbound,
        Capability::MemoryRead,
        Capability::MemoryWrite,
    ]);

    c.bench_function("capability_check_pass", |b| {
        b.iter(|| black_box(&caps).check(black_box(&[Capability::FsRead, Capability::NetOutbound])))
    });

    c.bench_function("capability_check_fail", |b| {
        b.iter(|| black_box(&caps).check(black_box(&[Capability::FsRead, Capability::ProcessExec])))
    });
}

fn bench_audit_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("audit_write");

    for count in [10, 100, 500] {
        group.bench_with_input(BenchmarkId::new("entries", count), &count, |b, &count| {
            b.iter_with_setup(
                || {
                    let tmp = TempDir::new().unwrap();
                    let path = tmp.path().join("audit.jsonl");
                    (AuditLog::new(path, true), tmp)
                },
                |(mut log, _tmp)| {
                    for i in 0..count {
                        log.log_tool_call(
                            &format!("tool_{i}"),
                            &format!("{{\"arg\": \"{i}\"}}"),
                            &format!("result_{i}"),
                            false,
                        );
                    }
                },
            );
        });
    }

    group.finish();
}

fn bench_audit_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("audit_verify");

    for count in [50, 200, 1000] {
        group.bench_with_input(BenchmarkId::new("entries", count), &count, |b, &count| {
            b.iter_with_setup(
                || {
                    let tmp = TempDir::new().unwrap();
                    let path = tmp.path().join("audit.jsonl");
                    let mut log = AuditLog::new(path.clone(), true);
                    for i in 0..count {
                        log.log_tool_call(
                            &format!("tool_{i}"),
                            &format!("{{\"n\": {i}}}"),
                            &format!("ok_{i}"),
                            false,
                        );
                    }
                    (AuditLog::new(path, true), tmp)
                },
                |(log, _tmp)| {
                    let result = log.verify().unwrap();
                    assert!(result.valid);
                    black_box(result.entries);
                },
            );
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_capability_check,
    bench_audit_write,
    bench_audit_verify,
);
criterion_main!(benches);
