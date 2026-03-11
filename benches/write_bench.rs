//! Writing benchmarks – compare `dxf` vs `acadrust` DXF writing performance.

use acadrust_dxf_bench::generators::{self, Scale};
use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};

// ---------------------------------------------------------------------------
// Bench: write to memory
// ---------------------------------------------------------------------------

fn bench_write_to_memory(c: &mut Criterion) {
    let counts: &[(Scale, &str)] = &[
        (Scale::Small, "small_100"),
        (Scale::Medium, "medium_1k"),
        (Scale::Large, "large_10k"),
        (Scale::Huge, "huge_100k"),
    ];

    // --- Lines only ---
    {
        let mut group = c.benchmark_group("write/memory/lines");
        for &(scale, label) in counts {
            let n = scale.count();
            group.throughput(Throughput::Elements(n as u64));

            group.bench_function(BenchmarkId::new("dxf-rs", label), |b| {
                let drawing = generators::build_dxf_lines(n);
                b.iter(|| {
                    let mut buf = Vec::with_capacity(n * 120);
                    drawing.save(&mut buf).expect("dxf save");
                    buf
                })
            });

            group.bench_function(BenchmarkId::new("acadrust", label), |b| {
                let doc = generators::build_acadrust_lines(n);
                b.iter(|| {
                    let writer = acadrust::DxfWriter::new(&doc);
                    writer.write_to_vec().expect("acadrust write_to_vec")
                })
            });
        }
        group.finish();
    }

    // --- Mixed entities ---
    {
        let mut group = c.benchmark_group("write/memory/mixed");
        for &(scale, label) in counts {
            let n = scale.count();
            group.throughput(Throughput::Elements(n as u64));

            group.bench_function(BenchmarkId::new("dxf-rs", label), |b| {
                let drawing = generators::build_dxf_mixed(n);
                b.iter(|| {
                    let mut buf = Vec::with_capacity(n * 200);
                    drawing.save(&mut buf).expect("dxf save");
                    buf
                })
            });

            group.bench_function(BenchmarkId::new("acadrust", label), |b| {
                let doc = generators::build_acadrust_mixed(n);
                b.iter(|| {
                    let writer = acadrust::DxfWriter::new(&doc);
                    writer.write_to_vec().expect("acadrust write_to_vec")
                })
            });
        }
        group.finish();
    }
}

// ---------------------------------------------------------------------------
// Bench: write to file on disk
// ---------------------------------------------------------------------------

fn bench_write_to_file(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("tmpdir");
    let n = Scale::Large.count();

    let mut group = c.benchmark_group("write/file_io");
    group.throughput(Throughput::Elements(n as u64));

    // Lines
    {
        let dxf_path = dir.path().join("dxf_out.dxf");
        group.bench_function("dxf-rs/lines", |b| {
            let drawing = generators::build_dxf_lines(n);
            b.iter(|| {
                drawing
                    .save_file(dxf_path.to_str().unwrap())
                    .expect("dxf save_file");
            })
        });

        let acad_path = dir.path().join("acadrust_out.dxf");
        group.bench_function("acadrust/lines", |b| {
            let doc = generators::build_acadrust_lines(n);
            b.iter(|| {
                let writer = acadrust::DxfWriter::new(&doc);
                writer
                    .write_to_file(acad_path.to_str().unwrap())
                    .expect("acadrust write_to_file");
            })
        });
    }

    // Mixed
    {
        let dxf_path = dir.path().join("dxf_mixed_out.dxf");
        group.bench_function("dxf-rs/mixed", |b| {
            let drawing = generators::build_dxf_mixed(n);
            b.iter(|| {
                drawing
                    .save_file(dxf_path.to_str().unwrap())
                    .expect("dxf save_file");
            })
        });

        let acad_path = dir.path().join("acadrust_mixed_out.dxf");
        group.bench_function("acadrust/mixed", |b| {
            let doc = generators::build_acadrust_mixed(n);
            b.iter(|| {
                let writer = acadrust::DxfWriter::new(&doc);
                writer
                    .write_to_file(acad_path.to_str().unwrap())
                    .expect("acadrust write_to_file");
            })
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Bench: entity-count scaling (write lines, varying N)
// ---------------------------------------------------------------------------

fn bench_write_scaling(c: &mut Criterion) {
    let counts = [100, 500, 1_000, 5_000, 10_000, 50_000];
    let mut group = c.benchmark_group("write/scaling_lines");

    for &n in &counts {
        group.throughput(Throughput::Elements(n as u64));

        group.bench_function(BenchmarkId::new("dxf-rs", n), |b| {
            let drawing = generators::build_dxf_lines(n);
            b.iter(|| {
                let mut buf = Vec::with_capacity(n * 120);
                drawing.save(&mut buf).expect("dxf save");
                buf
            })
        });

        group.bench_function(BenchmarkId::new("acadrust", n), |b| {
            let doc = generators::build_acadrust_lines(n);
            b.iter(|| {
                let writer = acadrust::DxfWriter::new(&doc);
                writer.write_to_vec().expect("acadrust write_to_vec")
            })
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion groups
// ---------------------------------------------------------------------------

criterion_group! {
    name = write_benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_secs(2))
        .measurement_time(std::time::Duration::from_secs(5));
    targets = bench_write_to_memory, bench_write_to_file, bench_write_scaling
}

criterion_main!(write_benches);
