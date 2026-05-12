//! End-to-end archive read throughput. Measures the full pipeline:
//! Archive::parse → ArchiveReader::extract.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use sevenzippy::Archive;

mod common;

fn bench_read_copy_only_19b(c: &mut Criterion) {
    let archive_bytes = common::load_fixture("copy_only.7z");
    let mut group = c.benchmark_group("archive_read/copy_only_19b");
    group.throughput(Throughput::Bytes(19));
    group.bench_function("parse+extract", |b| {
        b.iter(|| {
            let archive = Archive::parse(black_box(&archive_bytes)).unwrap();
            let extracted = archive.reader().extract(0).unwrap();
            black_box(extracted);
        });
    });
    group.finish();
}

fn bench_read_copy_only_64k(c: &mut Criterion) {
    let archive_bytes = common::load_fixture("copy_only_64k.7z");
    let mut group = c.benchmark_group("archive_read/copy_only_64k");
    group.throughput(Throughput::Bytes(65_536));
    group.bench_function("parse+extract", |b| {
        b.iter(|| {
            let archive = Archive::parse(black_box(&archive_bytes)).unwrap();
            let extracted = archive.reader().extract(0).unwrap();
            black_box(extracted);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_read_copy_only_19b, bench_read_copy_only_64k);
criterion_main!(benches);
