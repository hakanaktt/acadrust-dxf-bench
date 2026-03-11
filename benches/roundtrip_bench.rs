//! Roundtrip benchmarks – parse then re-write through each library.

use acadrust_dxf_bench::generators::{self, Scale};
use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::io::Cursor;

// ---------------------------------------------------------------------------
// Bench: full roundtrip  (read → write-to-memory)
// ---------------------------------------------------------------------------

fn bench_roundtrip(c: &mut Criterion) {
    let scales = [Scale::Medium, Scale::Large];

    for scale in &scales {
        let mixed_data = generators::generate_mixed(*scale);
        let lines_data = generators::generate_lines_only(*scale);

        let mut group = c.benchmark_group(format!("roundtrip/{}", scale.label()));

        // -- Mixed --
        group.throughput(Throughput::Bytes(mixed_data.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("dxf-rs", "mixed"),
            &mixed_data,
            |b, data| {
                b.iter(|| {
                    let mut cur = Cursor::new(data.as_slice());
                    let drawing = dxf::Drawing::load(&mut cur).expect("dxf parse");
                    let mut out = Vec::with_capacity(data.len());
                    drawing.save(&mut out).expect("dxf save");
                    out
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("acadrust", "mixed"),
            &mixed_data,
            |b, data| {
                b.iter(|| {
                    let cur = Cursor::new(data.clone());
                    let doc = acadrust::DxfReader::from_reader(cur)
                        .expect("acadrust reader")
                        .read()
                        .expect("acadrust parse");
                    let writer = acadrust::DxfWriter::new(&doc);
                    writer.write_to_vec().expect("acadrust write")
                })
            },
        );

        // -- Lines --
        group.throughput(Throughput::Bytes(lines_data.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("dxf-rs", "lines"),
            &lines_data,
            |b, data| {
                b.iter(|| {
                    let mut cur = Cursor::new(data.as_slice());
                    let drawing = dxf::Drawing::load(&mut cur).expect("dxf parse");
                    let mut out = Vec::with_capacity(data.len());
                    drawing.save(&mut out).expect("dxf save");
                    out
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("acadrust", "lines"),
            &lines_data,
            |b, data| {
                b.iter(|| {
                    let cur = Cursor::new(data.clone());
                    let doc = acadrust::DxfReader::from_reader(cur)
                        .expect("acadrust reader")
                        .read()
                        .expect("acadrust parse");
                    let writer = acadrust::DxfWriter::new(&doc);
                    writer.write_to_vec().expect("acadrust write")
                })
            },
        );

        group.finish();
    }
}

// ---------------------------------------------------------------------------
// Bench: cross-library roundtrip (parse with A → write with B)
// ---------------------------------------------------------------------------

fn bench_cross_library(c: &mut Criterion) {
    let data = generators::generate_mixed(Scale::Large);
    let mut group = c.benchmark_group("roundtrip/cross_library");
    group.throughput(Throughput::Bytes(data.len() as u64));

    // dxf-rs parse → dxf-rs write (baseline)
    group.bench_with_input(
        BenchmarkId::new("dxf_parse+dxf_write", "mixed_10k"),
        &data,
        |b, data| {
            b.iter(|| {
                let mut cur = Cursor::new(data.as_slice());
                let drawing = dxf::Drawing::load(&mut cur).unwrap();
                let mut out = Vec::with_capacity(data.len());
                drawing.save(&mut out).unwrap();
                out
            })
        },
    );

    // acadrust parse → acadrust write (baseline)
    group.bench_with_input(
        BenchmarkId::new("acad_parse+acad_write", "mixed_10k"),
        &data,
        |b, data| {
            b.iter(|| {
                let cur = Cursor::new(data.clone());
                let doc = acadrust::DxfReader::from_reader(cur)
                    .unwrap()
                    .read()
                    .unwrap();
                let writer = acadrust::DxfWriter::new(&doc);
                writer.write_to_vec().unwrap()
            })
        },
    );

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion groups
// ---------------------------------------------------------------------------

criterion_group! {
    name = roundtrip_benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_secs(2))
        .measurement_time(std::time::Duration::from_secs(5));
    targets = bench_roundtrip, bench_cross_library
}

criterion_main!(roundtrip_benches);
