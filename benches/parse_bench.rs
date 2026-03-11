//! Parsing benchmarks – compare `dxf` vs `acadrust` DXF reading performance.

use acadrust_dxf_bench::generators::{self, Scale};
use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::io::Cursor;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_with_dxf(data: &[u8]) {
    let mut cursor = Cursor::new(data);
    let _drawing = dxf::Drawing::load(&mut cursor).expect("dxf parse failed");
}

fn parse_with_acadrust(data: &[u8]) {
    let cursor = Cursor::new(data.to_vec());
    let _doc = acadrust::DxfReader::from_reader(cursor)
        .expect("acadrust reader init failed")
        .read()
        .expect("acadrust parse failed");
}

// ---------------------------------------------------------------------------
// Bench: parse by entity type at each scale
// ---------------------------------------------------------------------------

fn bench_parse_by_type(c: &mut Criterion) {
    let scales = [Scale::Small, Scale::Medium, Scale::Large, Scale::Huge];

    for scale in &scales {
        let variants = generators::all_variants(*scale);
        let mut group = c.benchmark_group(format!("parse/{}", scale.label()));

        for (variant_name, data) in &variants {
            let size = data.len() as u64;
            group.throughput(Throughput::Bytes(size));

            group.bench_with_input(
                BenchmarkId::new("dxf-rs", variant_name),
                data,
                |b, data| b.iter(|| parse_with_dxf(data)),
            );

            group.bench_with_input(
                BenchmarkId::new("acadrust", variant_name),
                data,
                |b, data| b.iter(|| parse_with_acadrust(data)),
            );
        }

        group.finish();
    }
}

// ---------------------------------------------------------------------------
// Bench: parse from file on disk
// ---------------------------------------------------------------------------

fn bench_parse_from_file(c: &mut Criterion) {
    let scale = Scale::Large;
    let data = generators::generate_mixed(scale);

    let dir = tempfile::tempdir().expect("tmpdir");
    let path = dir.path().join("bench_input.dxf");
    std::fs::write(&path, &data).expect("write tmp");

    let mut group = c.benchmark_group("parse/file_io");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("dxf-rs", |b| {
        b.iter(|| {
            let _d = dxf::Drawing::load_file(path.to_str().unwrap())
                .expect("dxf load_file");
        })
    });

    group.bench_function("acadrust", |b| {
        b.iter(|| {
            let _d = acadrust::DxfReader::from_file(path.to_str().unwrap())
                .expect("acadrust from_file")
                .read()
                .expect("acadrust read");
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Bench: entity-count scaling (lines only, varying N)
// ---------------------------------------------------------------------------

fn bench_parse_scaling(c: &mut Criterion) {
    let counts = [100, 500, 1_000, 5_000, 10_000, 50_000];
    let mut group = c.benchmark_group("parse/scaling_lines");

    for &n in &counts {
        let data = {
            let mut drawing = dxf::Drawing::new();
            let mut rng = rand::thread_rng();
            use rand::Rng;
            for _ in 0..n {
                let line = dxf::entities::Line {
                    p1: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    p2: dxf::Point::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                        0.0,
                    ),
                    ..Default::default()
                };
                drawing.add_entity(dxf::entities::Entity::new(
                    dxf::entities::EntityType::Line(line),
                ));
            }
            let mut buf = Vec::new();
            drawing.save(&mut buf).unwrap();
            buf
        };

        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("dxf-rs", n), &data, |b, data| {
            b.iter(|| parse_with_dxf(data))
        });

        group.bench_with_input(BenchmarkId::new("acadrust", n), &data, |b, data| {
            b.iter(|| parse_with_acadrust(data))
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion groups
// ---------------------------------------------------------------------------

criterion_group! {
    name = parse_benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_secs(2))
        .measurement_time(std::time::Duration::from_secs(5));
    targets = bench_parse_by_type, bench_parse_from_file, bench_parse_scaling
}

criterion_main!(parse_benches);
