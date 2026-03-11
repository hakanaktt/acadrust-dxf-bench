//! CLI runner for quick smoke-test comparisons outside of Criterion.
//!
//! Usage:
//!   cargo run --release -- [--scale small|medium|large|huge]

use acadrust_dxf_bench::generators::{self, Scale};
use clap::Parser;
use comfy_table::{Cell, ContentArrangement, Table};
use std::io::Cursor;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "dxf-bench", about = "Quick DXF benchmark comparison")]
struct Cli {
    /// Scale preset: small (100), medium (1k), large (10k), huge (100k)
    #[arg(long, default_value = "large")]
    scale: String,

    /// Number of iterations for timing
    #[arg(long, default_value_t = 5)]
    iterations: usize,
}

fn resolve_scale(s: &str) -> Scale {
    match s {
        "small" => Scale::Small,
        "medium" => Scale::Medium,
        "large" => Scale::Large,
        "huge" => Scale::Huge,
        "extrahuge" => Scale::ExtraHuge,
        _ => {
            eprintln!("Unknown scale '{}', using 'large'", s);
            Scale::Large
        }
    }
}

struct TimingResult {
    label: String,
    dxf_ms: f64,
    acadrust_ms: f64,
}

impl TimingResult {
    fn ratio(&self) -> f64 {
        if self.acadrust_ms > 0.0 {
            self.dxf_ms / self.acadrust_ms
        } else {
            f64::NAN
        }
    }
}

fn time_parse(data: &[u8], iterations: usize) -> (f64, f64) {
    // dxf-rs
    let start = Instant::now();
    for _ in 0..iterations {
        let mut cur = Cursor::new(data);
        let _d = dxf::Drawing::load(&mut cur).expect("dxf parse");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    // acadrust
    let start = Instant::now();
    for _ in 0..iterations {
        let cur = Cursor::new(data.to_vec());
        let _d = acadrust::DxfReader::from_reader(cur)
            .expect("acadrust reader")
            .read()
            .expect("acadrust parse");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

fn time_write_lines(n: usize, iterations: usize) -> (f64, f64) {
    let drawing = generators::build_dxf_lines(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let mut buf = Vec::with_capacity(n * 120);
        drawing.save(&mut buf).expect("dxf save");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    let doc = generators::build_acadrust_lines(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let writer = acadrust::DxfWriter::new(&doc);
        writer.write_to_vec().expect("acadrust write");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

fn time_write_mixed(n: usize, iterations: usize) -> (f64, f64) {
    let drawing = generators::build_dxf_mixed(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let mut buf = Vec::with_capacity(n * 200);
        drawing.save(&mut buf).expect("dxf save");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    let doc = generators::build_acadrust_mixed(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let writer = acadrust::DxfWriter::new(&doc);
        writer.write_to_vec().expect("acadrust write");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

fn time_write_binary_lines(n: usize, iterations: usize) -> (f64, f64) {
    let drawing = generators::build_dxf_lines(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let mut buf = Vec::with_capacity(n * 80);
        drawing.save_binary(&mut buf).expect("dxf save_binary");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    let doc = generators::build_acadrust_lines(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let writer = acadrust::DxfWriter::new_binary(&doc);
        writer.write_to_vec().expect("acadrust binary write");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

fn time_write_binary_mixed(n: usize, iterations: usize) -> (f64, f64) {
    let drawing = generators::build_dxf_mixed(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let mut buf = Vec::with_capacity(n * 120);
        drawing.save_binary(&mut buf).expect("dxf save_binary");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    let doc = generators::build_acadrust_mixed(n);
    let start = Instant::now();
    for _ in 0..iterations {
        let writer = acadrust::DxfWriter::new_binary(&doc);
        writer.write_to_vec().expect("acadrust binary write");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

fn main() {
    let cli = Cli::parse();
    let scale = resolve_scale(&cli.scale);
    let n = scale.count();
    let iters = cli.iterations;

    println!(
        "\n=== DXF Benchmark: dxf-rs vs acadrust  (scale={}, entities={}, iterations={}) ===\n",
        cli.scale, n, iters
    );

    // -----------------------------------------------------------------------
    // PARSE benchmarks
    // -----------------------------------------------------------------------
    println!("Generating test DXF files...");
    let variants = generators::all_variants(scale);
    let file_sizes: Vec<_> = variants
        .iter()
        .map(|(name, data)| (*name, data.len()))
        .collect();

    println!("Test files generated:");
    for (name, size) in &file_sizes {
        println!("  {:<20} {:>10} bytes", name, size);
    }
    println!();

    let mut parse_results = Vec::new();
    for (name, data) in &variants {
        let (dxf_ms, acad_ms) = time_parse(data, iters);
        parse_results.push(TimingResult {
            label: name.to_string(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
    }

    print_table("PARSE (from memory)", &parse_results);

    // -----------------------------------------------------------------------
    // WRITE benchmarks
    // -----------------------------------------------------------------------
    let mut write_results = Vec::new();

    let (dxf_ms, acad_ms) = time_write_lines(n, iters);
    write_results.push(TimingResult {
        label: "lines_only".into(),
        dxf_ms,
        acadrust_ms: acad_ms,
    });

    let (dxf_ms, acad_ms) = time_write_mixed(n, iters);
    write_results.push(TimingResult {
        label: "mixed".into(),
        dxf_ms,
        acadrust_ms: acad_ms,
    });

    print_table("WRITE (to memory)", &write_results);

    // -----------------------------------------------------------------------
    // ROUNDTRIP benchmarks
    // -----------------------------------------------------------------------
    let mixed_data = generators::generate_mixed(scale);
    let mut rt_results = Vec::new();

    // dxf roundtrip
    let start = Instant::now();
    for _ in 0..iters {
        let mut cur = Cursor::new(mixed_data.as_slice());
        let drawing = dxf::Drawing::load(&mut cur).unwrap();
        let mut out = Vec::with_capacity(mixed_data.len());
        drawing.save(&mut out).unwrap();
    }
    let dxf_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

    // acadrust roundtrip
    let start = Instant::now();
    for _ in 0..iters {
        let cur = Cursor::new(mixed_data.clone());
        let doc = acadrust::DxfReader::from_reader(cur)
            .unwrap()
            .read()
            .unwrap();
        let writer = acadrust::DxfWriter::new(&doc);
        writer.write_to_vec().unwrap();
    }
    let acad_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

    rt_results.push(TimingResult {
        label: "mixed_roundtrip".into(),
        dxf_ms: dxf_rt,
        acadrust_ms: acad_rt,
    });

    print_table("ROUNDTRIP (parse + write)", &rt_results);

    // -----------------------------------------------------------------------
    // BINARY DXF benchmarks
    // -----------------------------------------------------------------------
    println!("Generating binary DXF test files...");
    let binary_mixed = generators::generate_mixed_binary(scale);
    let binary_lines = generators::generate_lines_binary(scale);
    println!(
        "  binary_mixed       {:>10} bytes",
        binary_mixed.len()
    );
    println!(
        "  binary_lines       {:>10} bytes",
        binary_lines.len()
    );
    println!();

    // Binary parse
    let mut binary_parse_results = Vec::new();
    {
        let (dxf_ms, acad_ms) = time_parse(&binary_mixed, iters);
        binary_parse_results.push(TimingResult {
            label: "binary_mixed".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
        let (dxf_ms, acad_ms) = time_parse(&binary_lines, iters);
        binary_parse_results.push(TimingResult {
            label: "binary_lines".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
    }
    print_table("BINARY PARSE (from memory)", &binary_parse_results);

    // Binary write
    let mut binary_write_results = Vec::new();
    {
        let (dxf_ms, acad_ms) = time_write_binary_lines(n, iters);
        binary_write_results.push(TimingResult {
            label: "binary_lines".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
        let (dxf_ms, acad_ms) = time_write_binary_mixed(n, iters);
        binary_write_results.push(TimingResult {
            label: "binary_mixed".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
    }
    print_table("BINARY WRITE (to memory)", &binary_write_results);

    // Binary roundtrip
    let mut binary_rt_results = Vec::new();
    {
        // dxf-rs binary roundtrip
        let start = Instant::now();
        for _ in 0..iters {
            let mut cur = Cursor::new(binary_mixed.as_slice());
            let drawing = dxf::Drawing::load(&mut cur).unwrap();
            let mut out = Vec::with_capacity(binary_mixed.len());
            drawing.save_binary(&mut out).unwrap();
        }
        let dxf_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

        // acadrust binary roundtrip
        let start = Instant::now();
        for _ in 0..iters {
            let cur = Cursor::new(binary_mixed.clone());
            let doc = acadrust::DxfReader::from_reader(cur)
                .unwrap()
                .read()
                .unwrap();
            let writer = acadrust::DxfWriter::new_binary(&doc);
            writer.write_to_vec().unwrap();
        }
        let acad_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

        binary_rt_results.push(TimingResult {
            label: "binary_mixed_roundtrip".into(),
            dxf_ms: dxf_rt,
            acadrust_ms: acad_rt,
        });
    }
    print_table("BINARY ROUNDTRIP (parse + write)", &binary_rt_results);

    // -----------------------------------------------------------------------
    // DWG benchmarks (acadrust only – dxf-rs has no DWG support)
    // -----------------------------------------------------------------------
    println!("Generating DWG test files...");
    let dwg_mixed = generators::generate_mixed_dwg(scale);
    let dwg_lines = generators::generate_lines_dwg(scale);
    println!(
        "  dwg_mixed          {:>10} bytes",
        dwg_mixed.len()
    );
    println!(
        "  dwg_lines          {:>10} bytes",
        dwg_lines.len()
    );
    println!();

    // DWG parse (acadrust only)
    let mut dwg_parse_results = Vec::new();
    for (label, data) in &[("dwg_mixed", &dwg_mixed), ("dwg_lines", &dwg_lines)] {
        let start = Instant::now();
        for _ in 0..iters {
            let cur = Cursor::new(data.to_vec());
            let mut reader = acadrust::DwgReader::from_stream(cur);
            let _doc = reader.read().expect("acadrust DWG parse");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_parse_results.push(TimingResult {
            label: label.to_string(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });
    }
    print_table("DWG PARSE (acadrust only)", &dwg_parse_results);

    // DWG write (acadrust only)
    let mut dwg_write_results = Vec::new();
    {
        let doc = generators::build_acadrust_lines(n);
        let start = Instant::now();
        for _ in 0..iters {
            let _ = acadrust::DwgWriter::write_to_vec(&doc).expect("DWG write");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_write_results.push(TimingResult {
            label: "dwg_lines".into(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });

        let doc = generators::build_acadrust_mixed(n);
        let start = Instant::now();
        for _ in 0..iters {
            let _ = acadrust::DwgWriter::write_to_vec(&doc).expect("DWG write");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_write_results.push(TimingResult {
            label: "dwg_mixed".into(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });
    }
    print_table("DWG WRITE (acadrust only)", &dwg_write_results);

    // DWG roundtrip (acadrust only)
    let mut dwg_rt_results = Vec::new();
    {
        let start = Instant::now();
        for _ in 0..iters {
            let cur = Cursor::new(dwg_mixed.clone());
            let mut reader = acadrust::DwgReader::from_stream(cur);
            let doc = reader.read().expect("DWG parse");
            let _ = acadrust::DwgWriter::write_to_vec(&doc).expect("DWG write");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_rt_results.push(TimingResult {
            label: "dwg_mixed_roundtrip".into(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });
    }
    print_table("DWG ROUNDTRIP (acadrust only)", &dwg_rt_results);
}

fn print_table(title: &str, results: &[TimingResult]) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new(title),
        Cell::new("dxf-rs (ms)"),
        Cell::new("acadrust (ms)"),
        Cell::new("ratio (dxf/acad)"),
        Cell::new("faster"),
    ]);

    for r in results {
        let ratio = r.ratio();
        let (dxf_str, ratio_str, faster) = if r.dxf_ms.is_nan() {
            ("n/a".to_string(), "n/a".to_string(), "acadrust only")
        } else {
            let faster = if ratio > 1.0 {
                "acadrust"
            } else if ratio < 1.0 {
                "dxf-rs"
            } else {
                "tie"
            };
            (format!("{:.2}", r.dxf_ms), format!("{:.2}x", ratio), faster)
        };
        table.add_row(vec![
            Cell::new(&r.label),
            Cell::new(dxf_str),
            Cell::new(format!("{:.2}", r.acadrust_ms)),
            Cell::new(ratio_str),
            Cell::new(faster),
        ]);
    }

    println!("\n{}\n", table);
}
