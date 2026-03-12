//! CLI runner for quick smoke-test comparisons outside of Criterion.
//!
//! All reads/writes go through disk files under `bench_output/<scale>/`.
//! Roundtrip output files are preserved for validation.
//!
//! Usage:
//!   cargo run --release -- [--scale small|medium|large|huge]

use acadrust_dxf_bench::generators::{self, Scale};
use clap::Parser;
use comfy_table::{Cell, ContentArrangement, Table};
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
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

/// Parse a DXF file from disk with both libraries.
fn time_parse_file(path: &Path, iterations: usize) -> (f64, f64) {
    // dxf-rs: read from disk each iteration
    let start = Instant::now();
    for _ in 0..iterations {
        let file = fs::File::open(path).expect("open for dxf parse");
        let mut reader = BufReader::new(file);
        let _d = dxf::Drawing::load(&mut reader).expect("dxf parse");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    // acadrust: read from disk each iteration
    let start = Instant::now();
    for _ in 0..iterations {
        let file = fs::File::open(path).expect("open for acadrust parse");
        let reader = BufReader::new(file);
        let _d = acadrust::DxfReader::from_reader(reader)
            .expect("acadrust reader")
            .read()
            .expect("acadrust parse");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

/// Write a dxf::Drawing and acadrust doc to disk files, return (dxf_ms, acad_ms).
fn time_write_to_disk(
    drawing: &dxf::Drawing,
    doc: &acadrust::CadDocument,
    out_dir: &Path,
    label: &str,
    iterations: usize,
) -> (f64, f64) {
    let dxf_path = out_dir.join(format!("write_{}_dxfrs.dxf", label));
    let acad_path = out_dir.join(format!("write_{}_acadrust.dxf", label));

    // dxf-rs
    let start = Instant::now();
    for _ in 0..iterations {
        let file = fs::File::create(&dxf_path).expect("create dxf write file");
        let mut writer = BufWriter::new(file);
        drawing.save(&mut writer).expect("dxf save to disk");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    // acadrust
    let start = Instant::now();
    for _ in 0..iterations {
        let file = fs::File::create(&acad_path).expect("create acadrust write file");
        let writer = BufWriter::new(file);
        let dxf_writer = acadrust::DxfWriter::new(doc);
        dxf_writer.write_to_writer(writer).expect("acadrust write to disk");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

/// Write binary DXF to disk files, return (dxf_ms, acad_ms).
fn time_write_binary_to_disk(
    drawing: &dxf::Drawing,
    doc: &acadrust::CadDocument,
    out_dir: &Path,
    label: &str,
    iterations: usize,
) -> (f64, f64) {
    let dxf_path = out_dir.join(format!("write_binary_{}_dxfrs.dxb", label));
    let acad_path = out_dir.join(format!("write_binary_{}_acadrust.dxb", label));

    // dxf-rs
    let start = Instant::now();
    for _ in 0..iterations {
        let file = fs::File::create(&dxf_path).expect("create dxf binary write file");
        let mut writer = BufWriter::new(file);
        drawing.save_binary(&mut writer).expect("dxf save_binary to disk");
    }
    let dxf_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    // acadrust
    let start = Instant::now();
    for _ in 0..iterations {
        let file = fs::File::create(&acad_path).expect("create acadrust binary write file");
        let writer = BufWriter::new(file);
        let dxf_writer = acadrust::DxfWriter::new_binary(doc);
        dxf_writer.write_to_writer(writer).expect("acadrust binary write to disk");
    }
    let acad_total = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;

    (dxf_total, acad_total)
}

fn main() {
    let cli = Cli::parse();
    let scale = resolve_scale(&cli.scale);
    let n = scale.count();
    let iters = cli.iterations;

    // Create output directory
    let out_dir = PathBuf::from("bench_output").join(scale.label());
    fs::create_dir_all(&out_dir).expect("create output dir");
    let rt_dir = out_dir.join("roundtrip");
    fs::create_dir_all(&rt_dir).expect("create roundtrip dir");

    println!(
        "\n=== DXF Benchmark: dxf-rs vs acadrust  (scale={}, entities={}, iterations={}) ===",
        cli.scale, n, iters
    );
    println!("Output directory: {}\n", out_dir.display());

    // -----------------------------------------------------------------------
    // Generate & save test DXF files to disk
    // -----------------------------------------------------------------------
    println!("Generating test DXF files to disk...");
    let variants = generators::all_variants(scale);
    let mut input_files: Vec<(String, PathBuf)> = Vec::new();

    for (name, data) in &variants {
        let path = out_dir.join(format!("{}.dxf", name));
        fs::write(&path, data).expect("write test DXF to disk");
        println!("  {:<20} {:>10} bytes  -> {}", name, data.len(), path.display());
        input_files.push((name.to_string(), path));
    }
    println!();

    // -----------------------------------------------------------------------
    // PARSE benchmarks (from disk)
    // -----------------------------------------------------------------------
    let mut parse_results = Vec::new();
    for (name, path) in &input_files {
        let (dxf_ms, acad_ms) = time_parse_file(path, iters);
        parse_results.push(TimingResult {
            label: name.clone(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
    }
    print_table("PARSE (from disk)", &parse_results);

    // -----------------------------------------------------------------------
    // WRITE benchmarks (to disk)
    // -----------------------------------------------------------------------
    let mut write_results = Vec::new();

    let drawing = generators::build_dxf_lines(n);
    let doc = generators::build_acadrust_lines(n);
    let (dxf_ms, acad_ms) = time_write_to_disk(&drawing, &doc, &out_dir, "lines", iters);
    write_results.push(TimingResult {
        label: "lines_only".into(),
        dxf_ms,
        acadrust_ms: acad_ms,
    });

    let drawing = generators::build_dxf_mixed(n);
    let doc = generators::build_acadrust_mixed(n);
    let (dxf_ms, acad_ms) = time_write_to_disk(&drawing, &doc, &out_dir, "mixed", iters);
    write_results.push(TimingResult {
        label: "mixed".into(),
        dxf_ms,
        acadrust_ms: acad_ms,
    });

    print_table("WRITE (to disk)", &write_results);

    // -----------------------------------------------------------------------
    // ROUNDTRIP benchmarks (read from disk, write to disk — files kept)
    // -----------------------------------------------------------------------
    let mixed_input = out_dir.join("mixed.dxf");
    let mut rt_results = Vec::new();

    // dxf-rs roundtrip
    let dxf_rt_path = rt_dir.join("rt_mixed_dxfrs.dxf");
    let start = Instant::now();
    for _ in 0..iters {
        let file = fs::File::open(&mixed_input).expect("open for dxf roundtrip");
        let mut reader = BufReader::new(file);
        let drawing = dxf::Drawing::load(&mut reader).unwrap();
        let out_file = fs::File::create(&dxf_rt_path).unwrap();
        let mut writer = BufWriter::new(out_file);
        drawing.save(&mut writer).unwrap();
    }
    let dxf_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

    // acadrust roundtrip
    let acad_rt_path = rt_dir.join("rt_mixed_acadrust.dxf");
    let start = Instant::now();
    for _ in 0..iters {
        let file = fs::File::open(&mixed_input).expect("open for acadrust roundtrip");
        let reader = BufReader::new(file);
        let doc = acadrust::DxfReader::from_reader(reader)
            .unwrap()
            .read()
            .unwrap();
        let out_file = fs::File::create(&acad_rt_path).unwrap();
        let writer = BufWriter::new(out_file);
        let dxf_writer = acadrust::DxfWriter::new(&doc);
        dxf_writer.write_to_writer(writer).unwrap();
    }
    let acad_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

    rt_results.push(TimingResult {
        label: "mixed_roundtrip".into(),
        dxf_ms: dxf_rt,
        acadrust_ms: acad_rt,
    });

    print_table("ROUNDTRIP (disk → disk)", &rt_results);
    println!("  Roundtrip files kept at:");
    println!("    {}", dxf_rt_path.display());
    println!("    {}", acad_rt_path.display());
    println!();

    // -----------------------------------------------------------------------
    // BINARY DXF benchmarks
    // -----------------------------------------------------------------------
    println!("Generating binary DXF test files to disk...");
    let binary_mixed_data = generators::generate_mixed_binary(scale);
    let binary_lines_data = generators::generate_lines_binary(scale);
    let binary_mixed_path = out_dir.join("binary_mixed.dxb");
    let binary_lines_path = out_dir.join("binary_lines.dxb");
    fs::write(&binary_mixed_path, &binary_mixed_data).expect("write binary mixed");
    fs::write(&binary_lines_path, &binary_lines_data).expect("write binary lines");
    println!(
        "  binary_mixed       {:>10} bytes  -> {}",
        binary_mixed_data.len(),
        binary_mixed_path.display()
    );
    println!(
        "  binary_lines       {:>10} bytes  -> {}",
        binary_lines_data.len(),
        binary_lines_path.display()
    );
    println!();

    // Binary parse (from disk)
    let mut binary_parse_results = Vec::new();
    {
        let (dxf_ms, acad_ms) = time_parse_file(&binary_mixed_path, iters);
        binary_parse_results.push(TimingResult {
            label: "binary_mixed".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
        let (dxf_ms, acad_ms) = time_parse_file(&binary_lines_path, iters);
        binary_parse_results.push(TimingResult {
            label: "binary_lines".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
    }
    print_table("BINARY PARSE (from disk)", &binary_parse_results);

    // Binary write (to disk)
    let mut binary_write_results = Vec::new();
    {
        let drawing = generators::build_dxf_lines(n);
        let doc = generators::build_acadrust_lines(n);
        let (dxf_ms, acad_ms) =
            time_write_binary_to_disk(&drawing, &doc, &out_dir, "lines", iters);
        binary_write_results.push(TimingResult {
            label: "binary_lines".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });

        let drawing = generators::build_dxf_mixed(n);
        let doc = generators::build_acadrust_mixed(n);
        let (dxf_ms, acad_ms) =
            time_write_binary_to_disk(&drawing, &doc, &out_dir, "mixed", iters);
        binary_write_results.push(TimingResult {
            label: "binary_mixed".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
        });
    }
    print_table("BINARY WRITE (to disk)", &binary_write_results);

    // Binary roundtrip (disk → disk, files kept)
    let mut binary_rt_results = Vec::new();
    {
        let dxf_rt_path = rt_dir.join("rt_binary_mixed_dxfrs.dxb");
        let start = Instant::now();
        for _ in 0..iters {
            let file = fs::File::open(&binary_mixed_path).expect("open binary for dxf rt");
            let mut reader = BufReader::new(file);
            let drawing = dxf::Drawing::load(&mut reader).unwrap();
            let out_file = fs::File::create(&dxf_rt_path).unwrap();
            let mut writer = BufWriter::new(out_file);
            drawing.save_binary(&mut writer).unwrap();
        }
        let dxf_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

        let acad_rt_path = rt_dir.join("rt_binary_mixed_acadrust.dxb");
        let start = Instant::now();
        for _ in 0..iters {
            let file = fs::File::open(&binary_mixed_path).expect("open binary for acadrust rt");
            let reader = BufReader::new(file);
            let doc = acadrust::DxfReader::from_reader(reader)
                .unwrap()
                .read()
                .unwrap();
            let out_file = fs::File::create(&acad_rt_path).unwrap();
            let writer = BufWriter::new(out_file);
            let dxf_writer = acadrust::DxfWriter::new_binary(&doc);
            dxf_writer.write_to_writer(writer).unwrap();
        }
        let acad_rt = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;

        binary_rt_results.push(TimingResult {
            label: "binary_mixed_roundtrip".into(),
            dxf_ms: dxf_rt,
            acadrust_ms: acad_rt,
        });

        println!("  Binary roundtrip files kept at:");
        println!("    {}", dxf_rt_path.display());
        println!("    {}", acad_rt_path.display());
        println!();
    }
    print_table("BINARY ROUNDTRIP (disk → disk)", &binary_rt_results);

    // -----------------------------------------------------------------------
    // DWG benchmarks (acadrust only – dxf-rs has no DWG support)
    // -----------------------------------------------------------------------
    println!("Generating DWG test files to disk...");
    let dwg_mixed_data = generators::generate_mixed_dwg(scale);
    let dwg_lines_data = generators::generate_lines_dwg(scale);
    let dwg_mixed_path = out_dir.join("mixed.dwg");
    let dwg_lines_path = out_dir.join("lines.dwg");
    fs::write(&dwg_mixed_path, &dwg_mixed_data).expect("write DWG mixed");
    fs::write(&dwg_lines_path, &dwg_lines_data).expect("write DWG lines");
    println!(
        "  dwg_mixed          {:>10} bytes  -> {}",
        dwg_mixed_data.len(),
        dwg_mixed_path.display()
    );
    println!(
        "  dwg_lines          {:>10} bytes  -> {}",
        dwg_lines_data.len(),
        dwg_lines_path.display()
    );
    println!();

    // DWG parse (from disk, acadrust only)
    let mut dwg_parse_results = Vec::new();
    for (label, path) in &[("dwg_mixed", &dwg_mixed_path), ("dwg_lines", &dwg_lines_path)] {
        let start = Instant::now();
        for _ in 0..iters {
            let file = fs::File::open(path).expect("open DWG for parse");
            let reader = BufReader::new(file);
            let mut dwg_reader = acadrust::DwgReader::from_stream(reader);
            let _doc = dwg_reader.read().expect("acadrust DWG parse");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_parse_results.push(TimingResult {
            label: label.to_string(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });
    }
    print_table("DWG PARSE (from disk, acadrust only)", &dwg_parse_results);

    // DWG write (to disk, acadrust only)
    let mut dwg_write_results = Vec::new();
    {
        let doc = generators::build_acadrust_lines(n);
        let dwg_write_path = out_dir.join("write_dwg_lines_acadrust.dwg");
        let start = Instant::now();
        for _ in 0..iters {
            let file = fs::File::create(&dwg_write_path).expect("create DWG write file");
            let writer = BufWriter::new(file);
            acadrust::DwgWriter::write_to_writer(writer, &doc).expect("DWG write to disk");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_write_results.push(TimingResult {
            label: "dwg_lines".into(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });

        let doc = generators::build_acadrust_mixed(n);
        let dwg_write_path = out_dir.join("write_dwg_mixed_acadrust.dwg");
        let start = Instant::now();
        for _ in 0..iters {
            let file = fs::File::create(&dwg_write_path).expect("create DWG write file");
            let writer = BufWriter::new(file);
            acadrust::DwgWriter::write_to_writer(writer, &doc).expect("DWG write to disk");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_write_results.push(TimingResult {
            label: "dwg_mixed".into(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });
    }
    print_table("DWG WRITE (to disk, acadrust only)", &dwg_write_results);

    // DWG roundtrip (disk → disk, files kept, acadrust only)
    let mut dwg_rt_results = Vec::new();
    {
        let dwg_rt_path = rt_dir.join("rt_mixed_acadrust.dwg");
        let start = Instant::now();
        for _ in 0..iters {
            let file = fs::File::open(&dwg_mixed_path).expect("open DWG for roundtrip");
            let reader = BufReader::new(file);
            let mut dwg_reader = acadrust::DwgReader::from_stream(reader);
            let doc = dwg_reader.read().expect("DWG parse");
            let out_file = fs::File::create(&dwg_rt_path).unwrap();
            let writer = BufWriter::new(out_file);
            acadrust::DwgWriter::write_to_writer(writer, &doc).expect("DWG write");
        }
        let acad_ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
        dwg_rt_results.push(TimingResult {
            label: "dwg_mixed_roundtrip".into(),
            dxf_ms: f64::NAN,
            acadrust_ms: acad_ms,
        });

        println!("  DWG roundtrip file kept at:");
        println!("    {}", dwg_rt_path.display());
        println!();
    }
    print_table("DWG ROUNDTRIP (disk → disk, acadrust only)", &dwg_rt_results);

    // Summary of all output files
    println!("=== All output files in: {} ===", out_dir.display());
    fn list_files_recursive(dir: &Path, prefix: &str) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    list_files_recursive(&path, prefix);
                } else {
                    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    println!("  {:>12} bytes  {}", size, path.display());
                }
            }
        }
    }
    list_files_recursive(&out_dir, "");
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
