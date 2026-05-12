//! End-to-end archive write throughput. ArchiveBuilder::add_copy_file → build().

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use sevenzippy::ArchiveBuilder;

mod common;

fn bench_write_copy_64k_random(c: &mut Criterion) {
    let input = common::random(0xC0FFEE, 65_536);
    let mut group = c.benchmark_group("archive_write/copy_64k_random");
    group.throughput(Throughput::Bytes(65_536));
    group.bench_function("build", |b| {
        b.iter(|| {
            let mut builder = ArchiveBuilder::new();
            builder.add_copy_file("payload.bin", black_box(input.clone()));
            black_box(builder.build().unwrap());
        });
    });
    group.finish();
}

fn bench_write_copy_64k_zeros(c: &mut Criterion) {
    let input = common::zeros(65_536);
    let mut group = c.benchmark_group("archive_write/copy_64k_zeros");
    group.throughput(Throughput::Bytes(65_536));
    group.bench_function("build", |b| {
        b.iter(|| {
            let mut builder = ArchiveBuilder::new();
            builder.add_copy_file("payload.bin", black_box(input.clone()));
            black_box(builder.build().unwrap());
        });
    });
    group.finish();
}

fn bench_write_copy_1mib_sequential(c: &mut Criterion) {
    let input = common::sequential(1_048_576);
    let mut group = c.benchmark_group("archive_write/copy_1mib_sequential");
    group.throughput(Throughput::Bytes(1_048_576));
    group.bench_function("build", |b| {
        b.iter(|| {
            let mut builder = ArchiveBuilder::new();
            builder.add_copy_file("payload.bin", black_box(input.clone()));
            black_box(builder.build().unwrap());
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_write_copy_64k_random,
    bench_write_copy_64k_zeros,
    bench_write_copy_1mib_sequential
);
criterion_main!(benches);
