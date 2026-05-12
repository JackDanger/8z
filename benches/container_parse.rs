//! Header/folder parsing throughput in isolation (no decode).

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use eightz::Archive;

mod common;

fn bench_parse_copy_only_19b(c: &mut Criterion) {
    let archive_bytes = common::load_fixture("copy_only.7z");
    let mut group = c.benchmark_group("container_parse/copy_only_19b");
    group.throughput(Throughput::Bytes(archive_bytes.len() as u64));
    group.bench_function("parse", |b| {
        b.iter(|| black_box(Archive::parse(black_box(&archive_bytes)).unwrap()));
    });
    group.finish();
}

fn bench_parse_copy_only_64k(c: &mut Criterion) {
    let archive_bytes = common::load_fixture("copy_only_64k.7z");
    let mut group = c.benchmark_group("container_parse/copy_only_64k");
    group.throughput(Throughput::Bytes(archive_bytes.len() as u64));
    group.bench_function("parse", |b| {
        b.iter(|| black_box(Archive::parse(black_box(&archive_bytes)).unwrap()));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_copy_only_19b,
    bench_parse_copy_only_64k
);
criterion_main!(benches);
