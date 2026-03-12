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
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Command;
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
    acadsharp_ms: f64,
    ezdxf_ms: f64,
}

#[derive(Debug, Deserialize)]
struct AcadSharpTimingEntry {
    #[serde(rename = "Label")]
    label: String,
    #[serde(rename = "Ms")]
    ms: f64,
}

type AcadSharpResults = HashMap<String, Vec<AcadSharpTimingEntry>>;

/// Results from the ezdxf (Python) benchmark – same JSON shape.
type EzdxfResults = HashMap<String, Vec<AcadSharpTimingEntry>>;

/// Run the ACadSharp (.NET) benchmark and collect results.
fn run_acadsharp_bench(out_dir: &Path, iterations: usize) -> Option<AcadSharpResults> {
    let bench_dir = PathBuf::from("acadsharp-bench");
    let abs_out = std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(out_dir))
        .unwrap_or_else(|| out_dir.to_path_buf());

    println!("Running ACadSharp (.NET) benchmarks...");
    let output = Command::new("dotnet")
        .arg("run")
        .arg("-c")
        .arg("Release")
        .arg("--project")
        .arg(&bench_dir)
        .arg("--")
        .arg("--dir")
        .arg(&abs_out)
        .arg("--iterations")
        .arg(iterations.to_string())
        .output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !stderr.is_empty() {
                eprintln!("  ACadSharp stderr: {}", stderr.trim());
            }
            if !o.status.success() {
                eprintln!("  ACadSharp benchmark exited with {}", o.status);
                return None;
            }
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Find the JSON line (last non-empty line)
            let json_line = stdout.lines().filter(|l| l.starts_with('{')).last();
            match json_line {
                Some(line) => match serde_json::from_str::<AcadSharpResults>(line) {
                    Ok(r) => {
                        println!("  ACadSharp benchmarks completed successfully.");
                        Some(r)
                    }
                    Err(e) => {
                        eprintln!("  Failed to parse ACadSharp JSON: {}", e);
                        None
                    }
                },
                None => {
                    eprintln!("  No JSON output from ACadSharp benchmark.");
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("  Failed to run ACadSharp benchmark (is dotnet installed?): {}", e);
            None
        }
    }
}

/// Run the ezdxf (Python) benchmark and collect results.
fn run_ezdxf_bench(out_dir: &Path, iterations: usize) -> Option<EzdxfResults> {
    let bench_script = PathBuf::from("ezdxf-bench").join("bench.py");
    let abs_out = std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(out_dir))
        .unwrap_or_else(|| out_dir.to_path_buf());

    println!("Running ezdxf (Python) benchmarks...");
    let output = Command::new("python")
        .arg(&bench_script)
        .arg("--dir")
        .arg(&abs_out)
        .arg("--iterations")
        .arg(iterations.to_string())
        .output();

    match output {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !stderr.is_empty() {
                eprintln!("  ezdxf stderr: {}", stderr.trim());
            }
            if !o.status.success() {
                eprintln!("  ezdxf benchmark exited with {}", o.status);
                return None;
            }
            let stdout = String::from_utf8_lossy(&o.stdout);
            let json_line = stdout.lines().filter(|l| l.starts_with('{')).last();
            match json_line {
                Some(line) => match serde_json::from_str::<EzdxfResults>(line) {
                    Ok(r) => {
                        println!("  ezdxf benchmarks completed successfully.");
                        Some(r)
                    }
                    Err(e) => {
                        eprintln!("  Failed to parse ezdxf JSON: {}", e);
                        None
                    }
                },
                None => {
                    eprintln!("  No JSON output from ezdxf benchmark.");
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("  Failed to run ezdxf benchmark (is python installed?): {}", e);
            None
        }
    }
}

/// Look up ACadSharp timing for a given label in a category.
fn lookup_acadsharp(results: &Option<AcadSharpResults>, category: &str, label: &str) -> f64 {
    results
        .as_ref()
        .and_then(|r| r.get(category))
        .and_then(|entries| entries.iter().find(|e| e.label == label))
        .map(|e| e.ms)
        .unwrap_or(f64::NAN)
}

/// Look up ezdxf timing for a given label in a category.
fn lookup_ezdxf(results: &Option<EzdxfResults>, category: &str, label: &str) -> f64 {
    results
        .as_ref()
        .and_then(|r| r.get(category))
        .and_then(|entries| entries.iter().find(|e| e.label == label))
        .map(|e| e.ms)
        .unwrap_or(f64::NAN)
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
        "\n=== DXF Benchmark: dxf-rs vs acadrust vs ACadSharp vs ezdxf  (scale={}, entities={}, iterations={}) ===",
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
    // Generate binary DXF test files to disk
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

    // -----------------------------------------------------------------------
    // Generate DWG test files to disk
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

    // -----------------------------------------------------------------------
    // Run ACadSharp (.NET) benchmarks on the same files
    // -----------------------------------------------------------------------
    let acadsharp = run_acadsharp_bench(&out_dir, iters);
    println!();

    // -----------------------------------------------------------------------
    // Run ezdxf (Python) benchmarks on the same files
    // -----------------------------------------------------------------------
    let ezdxf = run_ezdxf_bench(&out_dir, iters);
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
            acadsharp_ms: lookup_acadsharp(&acadsharp, "parse", name),
            ezdxf_ms: lookup_ezdxf(&ezdxf, "parse", name),
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
        acadsharp_ms: lookup_acadsharp(&acadsharp, "write", "lines_only"),
        ezdxf_ms: lookup_ezdxf(&ezdxf, "write", "lines_only"),
    });

    let drawing = generators::build_dxf_mixed(n);
    let doc = generators::build_acadrust_mixed(n);
    let (dxf_ms, acad_ms) = time_write_to_disk(&drawing, &doc, &out_dir, "mixed", iters);
    write_results.push(TimingResult {
        label: "mixed".into(),
        dxf_ms,
        acadrust_ms: acad_ms,
        acadsharp_ms: lookup_acadsharp(&acadsharp, "write", "mixed"),
        ezdxf_ms: lookup_ezdxf(&ezdxf, "write", "mixed"),
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
        acadsharp_ms: lookup_acadsharp(&acadsharp, "roundtrip", "mixed_roundtrip"),
        ezdxf_ms: lookup_ezdxf(&ezdxf, "roundtrip", "mixed_roundtrip"),
    });

    print_table("ROUNDTRIP (disk \u{2192} disk)", &rt_results);
    println!("  Roundtrip files kept at:");
    println!("    {}", dxf_rt_path.display());
    println!("    {}", acad_rt_path.display());
    println!();

    // -----------------------------------------------------------------------
    // BINARY DXF benchmarks
    // -----------------------------------------------------------------------

    // Binary parse (from disk)
    let mut binary_parse_results = Vec::new();
    {
        let (dxf_ms, acad_ms) = time_parse_file(&binary_mixed_path, iters);
        binary_parse_results.push(TimingResult {
            label: "binary_mixed".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
            acadsharp_ms: lookup_acadsharp(&acadsharp, "binary_parse", "binary_mixed"),
            ezdxf_ms: lookup_ezdxf(&ezdxf, "binary_parse", "binary_mixed"),
        });
        let (dxf_ms, acad_ms) = time_parse_file(&binary_lines_path, iters);
        binary_parse_results.push(TimingResult {
            label: "binary_lines".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
            acadsharp_ms: lookup_acadsharp(&acadsharp, "binary_parse", "binary_lines"),
            ezdxf_ms: lookup_ezdxf(&ezdxf, "binary_parse", "binary_lines"),
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
            acadsharp_ms: lookup_acadsharp(&acadsharp, "binary_write", "binary_lines"),
            ezdxf_ms: lookup_ezdxf(&ezdxf, "binary_write", "binary_lines"),
        });

        let drawing = generators::build_dxf_mixed(n);
        let doc = generators::build_acadrust_mixed(n);
        let (dxf_ms, acad_ms) =
            time_write_binary_to_disk(&drawing, &doc, &out_dir, "mixed", iters);
        binary_write_results.push(TimingResult {
            label: "binary_mixed".into(),
            dxf_ms,
            acadrust_ms: acad_ms,
            acadsharp_ms: lookup_acadsharp(&acadsharp, "binary_write", "binary_mixed"),
            ezdxf_ms: lookup_ezdxf(&ezdxf, "binary_write", "binary_mixed"),
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
            acadsharp_ms: lookup_acadsharp(&acadsharp, "binary_roundtrip", "binary_mixed_roundtrip"),
            ezdxf_ms: lookup_ezdxf(&ezdxf, "binary_roundtrip", "binary_mixed_roundtrip"),
        });

        println!("  Binary roundtrip files kept at:");
        println!("    {}", dxf_rt_path.display());
        println!("    {}", acad_rt_path.display());
        println!();
    }
    print_table("BINARY ROUNDTRIP (disk → disk)", &binary_rt_results);

    // -----------------------------------------------------------------------
    // DWG benchmarks – dxf-rs has no DWG support
    // -----------------------------------------------------------------------

    // DWG parse (from disk)
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
            acadsharp_ms: lookup_acadsharp(&acadsharp, "dwg_parse", label),
            ezdxf_ms: f64::NAN,
        });
    }
    print_table("DWG PARSE (from disk)", &dwg_parse_results);

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
            acadsharp_ms: lookup_acadsharp(&acadsharp, "dwg_write", "dwg_lines"),
            ezdxf_ms: f64::NAN,
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
            acadsharp_ms: lookup_acadsharp(&acadsharp, "dwg_write", "dwg_mixed"),
            ezdxf_ms: f64::NAN,
        });
    }
    print_table("DWG WRITE (to disk)", &dwg_write_results);

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
            acadsharp_ms: lookup_acadsharp(&acadsharp, "dwg_roundtrip", "dwg_mixed_roundtrip"),
            ezdxf_ms: f64::NAN,
        });

        println!("  DWG roundtrip file kept at:");
        println!("    {}", dwg_rt_path.display());
        println!();
    }
    print_table("DWG ROUNDTRIP (disk \u{2192} disk)", &dwg_rt_results);

    // -----------------------------------------------------------------------
    // PARSE-DETAIL comparison – what does each library extract from the file?
    // -----------------------------------------------------------------------
    print_parse_detail(&mixed_input);

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
        Cell::new("ACadSharp (ms)"),
        Cell::new("ezdxf (ms)"),
        Cell::new("fastest"),
    ]);

    for r in results {
        let fmt = |v: f64| {
            if v.is_nan() {
                "n/a".to_string()
            } else {
                format!("{:.2}", v)
            }
        };

        // Determine the fastest library
        let candidates: Vec<(&str, f64)> = [
            ("dxf-rs", r.dxf_ms),
            ("acadrust", r.acadrust_ms),
            ("ACadSharp", r.acadsharp_ms),
            ("ezdxf", r.ezdxf_ms),
        ]
        .into_iter()
        .filter(|(_, v)| !v.is_nan() && *v > 0.0)
        .collect();

        let fastest = candidates
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| *name)
            .unwrap_or("n/a");

        table.add_row(vec![
            Cell::new(&r.label),
            Cell::new(fmt(r.dxf_ms)),
            Cell::new(fmt(r.acadrust_ms)),
            Cell::new(fmt(r.acadsharp_ms)),
            Cell::new(fmt(r.ezdxf_ms)),
            Cell::new(fastest),
        ]);
    }

    println!("\n{}\n", table);
}

// ---------------------------------------------------------------------------
// Parse-detail comparison: what each library extracts from a DXF file
// ---------------------------------------------------------------------------

/// Count dxf-rs entity types from a Drawing.
fn count_dxf_entities(drawing: &dxf::Drawing) -> Vec<(String, usize)> {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for entity in drawing.entities() {
        let name = match &entity.specific {
            dxf::entities::EntityType::Line(_) => "LINE",
            dxf::entities::EntityType::Circle(_) => "CIRCLE",
            dxf::entities::EntityType::Arc(_) => "ARC",
            dxf::entities::EntityType::Ellipse(_) => "ELLIPSE",
            dxf::entities::EntityType::ModelPoint(_) => "POINT",
            dxf::entities::EntityType::LwPolyline(_) => "LWPOLYLINE",
            dxf::entities::EntityType::Polyline(_) => "POLYLINE",
            dxf::entities::EntityType::Text(_) => "TEXT",
            dxf::entities::EntityType::MText(_) => "MTEXT",
            dxf::entities::EntityType::Spline(_) => "SPLINE",
            dxf::entities::EntityType::Solid(_) => "SOLID",
            dxf::entities::EntityType::Face3D(_) => "3DFACE",
            dxf::entities::EntityType::Insert(_) => "INSERT",
            dxf::entities::EntityType::RotatedDimension(_) => "DIMENSION (Rotated)",
            dxf::entities::EntityType::RadialDimension(_) => "DIMENSION (Radial)",
            dxf::entities::EntityType::DiameterDimension(_) => "DIMENSION (Diameter)",
            dxf::entities::EntityType::AngularThreePointDimension(_) => "DIMENSION (Angular3P)",
            dxf::entities::EntityType::OrdinateDimension(_) => "DIMENSION (Ordinate)",
            dxf::entities::EntityType::Image(_) => "IMAGE",
            dxf::entities::EntityType::Ray(_) => "RAY",
            dxf::entities::EntityType::XLine(_) => "XLINE",
            dxf::entities::EntityType::Helix(_) => "HELIX",
            dxf::entities::EntityType::Leader(_) => "LEADER",
            dxf::entities::EntityType::MLine(_) => "MLINE",
            dxf::entities::EntityType::Solid3D(_) => "3DSOLID",
            dxf::entities::EntityType::Body(_) => "BODY",
            dxf::entities::EntityType::Region(_) => "REGION",
            dxf::entities::EntityType::Tolerance(_) => "TOLERANCE",
            dxf::entities::EntityType::Wipeout(_) => "WIPEOUT",
            _ => "OTHER",
        };
        *counts.entry(name.to_string()).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}

/// Count acadrust entity types from a CadDocument.
fn count_acadrust_entities(doc: &acadrust::CadDocument) -> Vec<(String, usize)> {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for entity in doc.entities() {
        let name = match entity {
            acadrust::entities::EntityType::Line(_) => "LINE",
            acadrust::entities::EntityType::Circle(_) => "CIRCLE",
            acadrust::entities::EntityType::Arc(_) => "ARC",
            acadrust::entities::EntityType::Ellipse(_) => "ELLIPSE",
            acadrust::entities::EntityType::Point(_) => "POINT",
            acadrust::entities::EntityType::LwPolyline(_) => "LWPOLYLINE",
            acadrust::entities::EntityType::Polyline(_) => "POLYLINE",
            acadrust::entities::EntityType::Polyline2D(_) => "POLYLINE2D",
            acadrust::entities::EntityType::Polyline3D(_) => "POLYLINE3D",
            acadrust::entities::EntityType::Text(_) => "TEXT",
            acadrust::entities::EntityType::MText(_) => "MTEXT",
            acadrust::entities::EntityType::Spline(_) => "SPLINE",
            acadrust::entities::EntityType::Solid(_) => "SOLID",
            acadrust::entities::EntityType::Face3D(_) => "3DFACE",
            acadrust::entities::EntityType::Insert(_) => "INSERT",
            acadrust::entities::EntityType::Dimension(_) => "DIMENSION",
            acadrust::entities::EntityType::Hatch(_) => "HATCH",
            acadrust::entities::EntityType::Ray(_) => "RAY",
            acadrust::entities::EntityType::XLine(_) => "XLINE",
            acadrust::entities::EntityType::Leader(_) => "LEADER",
            acadrust::entities::EntityType::MLine(_) => "MLINE",
            acadrust::entities::EntityType::Solid3D(_) => "3DSOLID",
            acadrust::entities::EntityType::Region(_) => "REGION",
            acadrust::entities::EntityType::Body(_) => "BODY",
            acadrust::entities::EntityType::Tolerance(_) => "TOLERANCE",
            acadrust::entities::EntityType::Wipeout(_) => "WIPEOUT",
            acadrust::entities::EntityType::Block(_) => "BLOCK",
            acadrust::entities::EntityType::BlockEnd(_) => "BLOCKEND",
            _ => "OTHER",
        };
        *counts.entry(name.to_string()).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}

/// Inspect common entity property coverage.
fn inspect_dxf_common(drawing: &dxf::Drawing) -> Vec<(String, usize, usize)> {
    let mut total = 0usize;
    let mut has_handle = 0usize;
    let mut has_layer = 0usize;
    let mut has_color = 0usize;
    let mut has_linetype = 0usize;
    let mut has_linetype_scale = 0usize;
    let mut has_lineweight = 0usize;
    let mut has_thickness = 0usize;

    for e in drawing.entities() {
        total += 1;
        let c = &e.common;
        if c.handle.0 != 0 {
            has_handle += 1;
        }
        if !c.layer.is_empty() && c.layer != "0" {
            has_layer += 1;
        }
        if c.color.index().is_some() {
            has_color += 1;
        }
        if !c.line_type_name.is_empty() && c.line_type_name != "BYLAYER" {
            has_linetype += 1;
        }
        if c.line_type_scale != 1.0 {
            has_linetype_scale += 1;
        }
        if c.lineweight_enum_value != 0 && c.lineweight_enum_value != -1 {
            has_lineweight += 1;
        }
        let thickness: f64 = match &e.specific {
            dxf::entities::EntityType::Line(l) => l.thickness,
            dxf::entities::EntityType::Circle(c) => c.thickness,
            dxf::entities::EntityType::Arc(a) => a.thickness,
            _ => 0.0,
        };
        if thickness != 0.0 {
            has_thickness += 1;
        }
    }

    vec![
        ("handle".into(), has_handle, total),
        ("layer (non-0)".into(), has_layer, total),
        ("color (non-BYLAYER)".into(), has_color, total),
        ("linetype (non-BYLAYER)".into(), has_linetype, total),
        ("linetype_scale (≠1.0)".into(), has_linetype_scale, total),
        ("lineweight (non-default)".into(), has_lineweight, total),
        ("thickness (≠0)".into(), has_thickness, total),
    ]
}

fn inspect_acadrust_common(doc: &acadrust::CadDocument) -> Vec<(String, usize, usize)> {
    let mut total = 0usize;
    let mut has_handle = 0usize;
    let mut has_layer = 0usize;
    let mut has_color = 0usize;
    let mut has_linetype = 0usize;
    let mut has_linetype_scale = 0usize;
    let mut has_lineweight = 0usize;
    let mut has_thickness = 0usize;

    for e in doc.entities() {
        total += 1;
        let c: &acadrust::entities::EntityCommon = e.common();
        if !c.handle.is_null() {
            has_handle += 1;
        }
        if !c.layer.is_empty() && c.layer != "0" {
            has_layer += 1;
        }
        if !matches!(c.color, acadrust::Color::ByLayer) {
            has_color += 1;
        }
        if !c.linetype.is_empty() && c.linetype != "BYLAYER" && c.linetype != "ByLayer" {
            has_linetype += 1;
        }
        if c.linetype_scale != 1.0 {
            has_linetype_scale += 1;
        }
        if !matches!(c.line_weight, acadrust::LineWeight::ByLayer) {
            has_lineweight += 1;
        }
        let thickness: f64 = match e {
            acadrust::entities::EntityType::Line(l) => l.thickness,
            acadrust::entities::EntityType::Circle(c) => c.thickness,
            acadrust::entities::EntityType::Arc(a) => a.thickness,
            _ => 0.0,
        };
        if thickness != 0.0 {
            has_thickness += 1;
        }
    }

    vec![
        ("handle".into(), has_handle, total),
        ("layer (non-0)".into(), has_layer, total),
        ("color (non-BYLAYER)".into(), has_color, total),
        ("linetype (non-BYLAYER)".into(), has_linetype, total),
        ("linetype_scale (≠1.0)".into(), has_linetype_scale, total),
        ("lineweight (non-default)".into(), has_lineweight, total),
        ("thickness (≠0)".into(), has_thickness, total),
    ]
}

/// Spot-check geometric values from the first entity of each type.
fn spot_check_line(drawing: &dxf::Drawing, doc: &acadrust::CadDocument) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();

    // Find first Line in dxf-rs
    let dxf_line = drawing.entities().find_map(|e| match &e.specific {
        dxf::entities::EntityType::Line(l) => Some(l),
        _ => None,
    });
    let acad_line = doc.entities().find_map(|e| match e {
        acadrust::entities::EntityType::Line(l) => Some(l),
        _ => None,
    });

    if let (Some(dl), Some(al)) = (dxf_line, acad_line) {
        rows.push(vec![
            "LINE.start".into(),
            format!("({:.4}, {:.4}, {:.4})", dl.p1.x, dl.p1.y, dl.p1.z),
            format!("({:.4}, {:.4}, {:.4})", al.start.x, al.start.y, al.start.z),
        ]);
        rows.push(vec![
            "LINE.end".into(),
            format!("({:.4}, {:.4}, {:.4})", dl.p2.x, dl.p2.y, dl.p2.z),
            format!("({:.4}, {:.4}, {:.4})", al.end.x, al.end.y, al.end.z),
        ]);
        rows.push(vec![
            "LINE.thickness".into(),
            format!("{:.4}", dl.thickness),
            format!("{:.4}", al.thickness),
        ]);
        rows.push(vec![
            "LINE.layer".into(),
            drawing.entities().find_map(|e| match &e.specific {
                dxf::entities::EntityType::Line(_) => Some(e.common.layer.clone()),
                _ => None,
            }).unwrap_or_default(),
            al.common.layer.clone(),
        ]);
    }

    // Find first Circle
    let dxf_circle = drawing.entities().find_map(|e| match &e.specific {
        dxf::entities::EntityType::Circle(c) => Some((c.clone(), e.common.layer.clone())),
        _ => None,
    });
    let acad_circle = doc.entities().find_map(|e| match e {
        acadrust::entities::EntityType::Circle(c) => Some(c),
        _ => None,
    });
    if let (Some((dc, dl)), Some(ac)) = (dxf_circle, acad_circle) {
        rows.push(vec![
            "CIRCLE.center".into(),
            format!("({:.4}, {:.4}, {:.4})", dc.center.x, dc.center.y, dc.center.z),
            format!("({:.4}, {:.4}, {:.4})", ac.center.x, ac.center.y, ac.center.z),
        ]);
        rows.push(vec![
            "CIRCLE.radius".into(),
            format!("{:.4}", dc.radius),
            format!("{:.4}", ac.radius),
        ]);
        rows.push(vec![
            "CIRCLE.layer".into(),
            dl,
            ac.common.layer.clone(),
        ]);
    }

    // Find first Arc
    let dxf_arc = drawing.entities().find_map(|e| match &e.specific {
        dxf::entities::EntityType::Arc(a) => Some((a.clone(), e.common.layer.clone())),
        _ => None,
    });
    let acad_arc = doc.entities().find_map(|e| match e {
        acadrust::entities::EntityType::Arc(a) => Some(a),
        _ => None,
    });
    if let (Some((da, dl)), Some(aa)) = (dxf_arc, acad_arc) {
        rows.push(vec![
            "ARC.center".into(),
            format!("({:.4}, {:.4}, {:.4})", da.center.x, da.center.y, da.center.z),
            format!("({:.4}, {:.4}, {:.4})", aa.center.x, aa.center.y, aa.center.z),
        ]);
        rows.push(vec![
            "ARC.radius".into(),
            format!("{:.4}", da.radius),
            format!("{:.4}", aa.radius),
        ]);
        rows.push(vec![
            "ARC.start_angle".into(),
            format!("{:.4}°", da.start_angle),
            format!("{:.4} rad ({:.4}°)", aa.start_angle, aa.start_angle.to_degrees()),
        ]);
        rows.push(vec![
            "ARC.end_angle".into(),
            format!("{:.4}°", da.end_angle),
            format!("{:.4} rad ({:.4}°)", aa.end_angle, aa.end_angle.to_degrees()),
        ]);
        rows.push(vec![
            "ARC.layer".into(),
            dl,
            aa.common.layer.clone(),
        ]);
    }

    // Find first Ellipse
    let dxf_ellipse = drawing.entities().find_map(|e| match &e.specific {
        dxf::entities::EntityType::Ellipse(el) => Some((el.clone(), e.common.layer.clone())),
        _ => None,
    });
    let acad_ellipse = doc.entities().find_map(|e| match e {
        acadrust::entities::EntityType::Ellipse(el) => Some(el),
        _ => None,
    });
    if let (Some((de, dl)), Some(ae)) = (dxf_ellipse, acad_ellipse) {
        rows.push(vec![
            "ELLIPSE.center".into(),
            format!("({:.4}, {:.4}, {:.4})", de.center.x, de.center.y, de.center.z),
            format!("({:.4}, {:.4}, {:.4})", ae.center.x, ae.center.y, ae.center.z),
        ]);
        rows.push(vec![
            "ELLIPSE.major_axis".into(),
            format!("({:.4}, {:.4}, {:.4})", de.major_axis.x, de.major_axis.y, de.major_axis.z),
            format!("({:.4}, {:.4}, {:.4})", ae.major_axis.x, ae.major_axis.y, ae.major_axis.z),
        ]);
        rows.push(vec![
            "ELLIPSE.axis_ratio".into(),
            format!("{:.4}", de.minor_axis_ratio),
            format!("{:.4}", ae.minor_axis_ratio),
        ]);
        rows.push(vec![
            "ELLIPSE.layer".into(),
            dl,
            ae.common.layer.clone(),
        ]);
    }

    rows
}

/// Parse the input file with both libraries and print a detailed comparison.
fn print_parse_detail(input_path: &Path) {
    println!("\n=== PARSE DETAIL: what each library extracts from {} ===\n",
        input_path.file_name().unwrap_or_default().to_string_lossy());

    // --- Parse with both libraries ---
    let dxf_drawing = {
        let file = fs::File::open(input_path).expect("open for dxf parse-detail");
        let mut reader = BufReader::new(file);
        dxf::Drawing::load(&mut reader).expect("dxf parse-detail")
    };
    let acad_doc = {
        let file = fs::File::open(input_path).expect("open for acadrust parse-detail");
        let reader = BufReader::new(file);
        acadrust::DxfReader::from_reader(reader)
            .expect("acadrust reader")
            .read()
            .expect("acadrust parse-detail")
    };

    // --- 1. Section / table summary ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Section / Table"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        table.add_row(vec![
            Cell::new("Layers"),
            Cell::new(dxf_drawing.layers().count()),
            Cell::new(acad_doc.layers.len()),
        ]);
        table.add_row(vec![
            Cell::new("LineTypes"),
            Cell::new(dxf_drawing.line_types().count()),
            Cell::new(acad_doc.line_types.len()),
        ]);
        table.add_row(vec![
            Cell::new("Text Styles"),
            Cell::new(dxf_drawing.styles().count()),
            Cell::new(acad_doc.text_styles.len()),
        ]);
        table.add_row(vec![
            Cell::new("DimStyles"),
            Cell::new(dxf_drawing.dim_styles().count()),
            Cell::new(acad_doc.dim_styles.len()),
        ]);
        table.add_row(vec![
            Cell::new("AppIds"),
            Cell::new(dxf_drawing.app_ids().count()),
            Cell::new(acad_doc.app_ids.len()),
        ]);
        table.add_row(vec![
            Cell::new("Block Records"),
            Cell::new(dxf_drawing.block_records().count()),
            Cell::new(acad_doc.block_records.len()),
        ]);
        table.add_row(vec![
            Cell::new("Views"),
            Cell::new(dxf_drawing.views().count()),
            Cell::new(acad_doc.views.len()),
        ]);
        table.add_row(vec![
            Cell::new("ViewPorts"),
            Cell::new(dxf_drawing.view_ports().count()),
            Cell::new(acad_doc.vports.len()),
        ]);
        table.add_row(vec![
            Cell::new("UCS"),
            Cell::new(dxf_drawing.ucss().count()),
            Cell::new(acad_doc.ucss.len()),
        ]);
        table.add_row(vec![
            Cell::new("Blocks (definitions)"),
            Cell::new(dxf_drawing.blocks().count()),
            Cell::new(acad_doc.block_records.len()),
        ]);
        table.add_row(vec![
            Cell::new("Entities (total)"),
            Cell::new(dxf_drawing.entities().count()),
            Cell::new(acad_doc.entities().count()),
        ]);
        table.add_row(vec![
            Cell::new("Objects"),
            Cell::new(dxf_drawing.objects().count()),
            Cell::new(acad_doc.objects.len()),
        ]);

        println!("{}\n", table);
    }

    // --- 2. Entity breakdown by type ---
    {
        let dxf_counts = count_dxf_entities(&dxf_drawing);
        let acad_counts = count_acadrust_entities(&acad_doc);

        let mut all_types: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for (k, _) in &dxf_counts { all_types.insert(k.clone()); }
        for (k, _) in &acad_counts { all_types.insert(k.clone()); }

        let dxf_map: std::collections::BTreeMap<String, usize> = dxf_counts.into_iter().collect();
        let acad_map: std::collections::BTreeMap<String, usize> = acad_counts.into_iter().collect();

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Entity Type"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
            Cell::new("match?"),
        ]);

        for typ in &all_types {
            let d = dxf_map.get(typ).copied().unwrap_or(0);
            let a = acad_map.get(typ).copied().unwrap_or(0);
            let status = if d == a { "✓" } else { "✗" };
            table.add_row(vec![
                Cell::new(typ),
                Cell::new(d),
                Cell::new(a),
                Cell::new(status),
            ]);
        }

        println!("Entity breakdown:\n{}\n", table);
    }

    // --- 3. Layer detail ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Layer"),
            Cell::new("dxf-rs color"),
            Cell::new("acadrust color"),
            Cell::new("dxf-rs linetype"),
            Cell::new("acadrust linetype"),
        ]);

        let mut dxf_layers: Vec<_> = dxf_drawing.layers().collect();
        dxf_layers.sort_by(|a, b| a.name.cmp(&b.name));

        for dl in &dxf_layers {
            let acad_layer = acad_doc.layers.iter().find(|l| l.name == dl.name);
            let (ac_color, ac_lt) = if let Some(al) = acad_layer {
                (format!("{:?}", al.color), al.line_type.clone())
            } else {
                ("—".into(), "—".into())
            };
            table.add_row(vec![
                Cell::new(&dl.name),
                Cell::new(format!("{:?}", dl.color.index())),
                Cell::new(ac_color),
                Cell::new(&dl.line_type_name),
                Cell::new(ac_lt),
            ]);
        }

        for al in acad_doc.layers.iter() {
            if !dxf_layers.iter().any(|dl| dl.name == al.name) {
                table.add_row(vec![
                    Cell::new(format!("{} (acadrust only)", al.name)),
                    Cell::new("—"),
                    Cell::new(format!("{:?}", al.color)),
                    Cell::new("—"),
                    Cell::new(&al.line_type),
                ]);
            }
        }

        println!("Layers:\n{}\n", table);
    }

    // --- 4. Common entity property coverage ---
    {
        let dxf_props = inspect_dxf_common(&dxf_drawing);
        let acad_props = inspect_acadrust_common(&acad_doc);

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Property"),
            Cell::new("dxf-rs (set/total)"),
            Cell::new("acadrust (set/total)"),
        ]);

        for (dp, ap) in dxf_props.iter().zip(acad_props.iter()) {
            table.add_row(vec![
                Cell::new(&dp.0),
                Cell::new(format!("{}/{}", dp.1, dp.2)),
                Cell::new(format!("{}/{}", ap.1, ap.2)),
            ]);
        }

        println!("Common entity properties coverage:\n{}\n", table);
    }

    // --- 5. Geometric value spot-check (first entity of each type) ---
    {
        let rows = spot_check_line(&dxf_drawing, &acad_doc);
        if !rows.is_empty() {
            let mut table = Table::new();
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec![
                Cell::new("Field"),
                Cell::new("dxf-rs value"),
                Cell::new("acadrust value"),
            ]);
            for row in &rows {
                table.add_row(vec![
                    Cell::new(&row[0]),
                    Cell::new(&row[1]),
                    Cell::new(&row[2]),
                ]);
            }
            println!("Geometric spot-check (1st entity of each type):\n{}\n", table);
        }
    }

    // --- 6. Line types detail ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("LineType"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        let mut dxf_lts: Vec<_> = dxf_drawing.line_types().map(|lt| lt.name.clone()).collect();
        dxf_lts.sort();
        let acad_lts: Vec<_> = acad_doc.line_types.iter().map(|lt| lt.name.clone()).collect();

        let mut all_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for n in &dxf_lts { all_names.insert(n.clone()); }
        for n in &acad_lts { all_names.insert(n.clone()); }

        for name in &all_names {
            let in_dxf = if dxf_lts.contains(name) { "✓" } else { "—" };
            let in_acad = if acad_lts.contains(name) { "✓" } else { "—" };
            table.add_row(vec![
                Cell::new(name),
                Cell::new(in_dxf),
                Cell::new(in_acad),
            ]);
        }

        println!("LineTypes:\n{}\n", table);
    }

    // --- 7. Text styles detail ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Text Style"),
            Cell::new("dxf-rs font"),
            Cell::new("acadrust font"),
        ]);

        for ds in dxf_drawing.styles() {
            let acad_style = acad_doc.text_styles.iter().find(|s| s.name == ds.name);
            let acad_font = acad_style
                .map(|s| s.font_file.clone())
                .unwrap_or_else(|| "—".into());
            table.add_row(vec![
                Cell::new(&ds.name),
                Cell::new(&ds.primary_font_file_name),
                Cell::new(acad_font),
            ]);
        }

        println!("Text Styles:\n{}\n", table);
    }

    // --- 8. Header variables comparison ---
    {
        let dh = &dxf_drawing.header;
        let ah = &acad_doc.header;

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Header Variable"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        table.add_row(vec![
            Cell::new("Version"),
            Cell::new(format!("{:?}", dh.version)),
            Cell::new(acad_doc.version.as_str()),
        ]);
        table.add_row(vec![
            Cell::new("HANDSEED"),
            Cell::new(format!("0x{:X}", dh.next_available_handle.0)),
            Cell::new(format!("0x{:X}", ah.handle_seed)),
        ]);
        table.add_row(vec![
            Cell::new("Current Layer"),
            Cell::new(&dh.current_layer),
            Cell::new(&ah.current_layer_name),
        ]);
        table.add_row(vec![
            Cell::new("Text Style"),
            Cell::new(&dh.text_style),
            Cell::new(&ah.current_text_style_name),
        ]);
        table.add_row(vec![
            Cell::new("DimStyle"),
            Cell::new(&dh.dimension_style_name),
            Cell::new(&ah.current_dimstyle_name),
        ]);
        table.add_row(vec![
            Cell::new("LTSCALE"),
            Cell::new(format!("{:.4}", dh.line_type_scale)),
            Cell::new(format!("{:.4}", ah.linetype_scale)),
        ]);
        table.add_row(vec![
            Cell::new("TEXTSIZE"),
            Cell::new(format!("{:.4}", dh.default_text_height)),
            Cell::new(format!("{:.4}", ah.text_height)),
        ]);
        table.add_row(vec![
            Cell::new("EXTMIN"),
            Cell::new(format!("({:.2}, {:.2}, {:.2})",
                dh.minimum_drawing_extents.x,
                dh.minimum_drawing_extents.y,
                dh.minimum_drawing_extents.z)),
            Cell::new(format!("({:.2}, {:.2}, {:.2})",
                ah.model_space_extents_min.x,
                ah.model_space_extents_min.y,
                ah.model_space_extents_min.z)),
        ]);
        table.add_row(vec![
            Cell::new("EXTMAX"),
            Cell::new(format!("({:.2}, {:.2}, {:.2})",
                dh.maximum_drawing_extents.x,
                dh.maximum_drawing_extents.y,
                dh.maximum_drawing_extents.z)),
            Cell::new(format!("({:.2}, {:.2}, {:.2})",
                ah.model_space_extents_max.x,
                ah.model_space_extents_max.y,
                ah.model_space_extents_max.z)),
        ]);
        table.add_row(vec![
            Cell::new("LIMMIN"),
            Cell::new(format!("({:.2}, {:.2})",
                dh.minimum_drawing_limits.x,
                dh.minimum_drawing_limits.y)),
            Cell::new(format!("({:.2}, {:.2})",
                ah.model_space_limits_min.x,
                ah.model_space_limits_min.y)),
        ]);
        table.add_row(vec![
            Cell::new("LIMMAX"),
            Cell::new(format!("({:.2}, {:.2})",
                dh.maximum_drawing_limits.x,
                dh.maximum_drawing_limits.y)),
            Cell::new(format!("({:.2}, {:.2})",
                ah.model_space_limits_max.x,
                ah.model_space_limits_max.y)),
        ]);
        table.add_row(vec![
            Cell::new("INSBASE"),
            Cell::new(format!("({:.2}, {:.2}, {:.2})",
                dh.insertion_base.x,
                dh.insertion_base.y,
                dh.insertion_base.z)),
            Cell::new(format!("({:.2}, {:.2}, {:.2})",
                ah.model_space_insertion_base.x,
                ah.model_space_insertion_base.y,
                ah.model_space_insertion_base.z)),
        ]);
        table.add_row(vec![
            Cell::new("FILLMODE"),
            Cell::new(format!("{}", dh.fill_mode_on)),
            Cell::new(format!("{}", ah.fill_mode)),
        ]);
        table.add_row(vec![
            Cell::new("ORTHOMODE"),
            Cell::new(format!("{}", dh.draw_orthogonal_lines)),
            Cell::new(format!("{}", ah.ortho_mode)),
        ]);
        table.add_row(vec![
            Cell::new("MIRRTEXT"),
            Cell::new(format!("{}", dh.mirror_text)),
            Cell::new(format!("{}", ah.mirror_text)),
        ]);
        table.add_row(vec![
            Cell::new("CECOLOR"),
            Cell::new(format!("{:?}", dh.current_entity_color.index())),
            Cell::new(format!("{:?}", ah.current_entity_color)),
        ]);
        table.add_row(vec![
            Cell::new("CELTSCALE"),
            Cell::new(format!("{:.4}", dh.current_entity_line_type_scale)),
            Cell::new(format!("{:.4}", ah.current_entity_linetype_scale)),
        ]);

        println!("Header variables:\n{}\n", table);
    }

    // --- 9. DimStyle detail ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("DimStyle Field"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        for dds in dxf_drawing.dim_styles() {
            let ads = acad_doc.dim_styles.iter().find(|s| s.name == dds.name);
            table.add_row(vec![
                Cell::new(format!("[{}]", dds.name)),
                Cell::new(""),
                Cell::new(if ads.is_some() { "✓" } else { "—" }),
            ]);
            if let Some(ads) = ads {
                table.add_row(vec![
                    Cell::new("  DIMSCALE"),
                    Cell::new(format!("{:.4}", dds.dimensioning_scale_factor)),
                    Cell::new(format!("{:.4}", ads.dimscale)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMASZ"),
                    Cell::new(format!("{:.4}", dds.dimensioning_arrow_size)),
                    Cell::new(format!("{:.4}", ads.dimasz)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMTXT"),
                    Cell::new(format!("{:.4}", dds.dimensioning_text_height)),
                    Cell::new(format!("{:.4}", ads.dimtxt)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMEXO"),
                    Cell::new(format!("{:.4}", dds.dimension_extension_line_offset)),
                    Cell::new(format!("{:.4}", ads.dimexo)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMEXE"),
                    Cell::new(format!("{:.4}", dds.dimension_extension_line_extension)),
                    Cell::new(format!("{:.4}", ads.dimexe)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMDLI"),
                    Cell::new(format!("{:.4}", dds.dimension_line_increment)),
                    Cell::new(format!("{:.4}", ads.dimdli)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMDLE"),
                    Cell::new(format!("{:.4}", dds.dimension_line_extension)),
                    Cell::new(format!("{:.4}", ads.dimdle)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMGAP"),
                    Cell::new(format!("{:.4}", dds.dimension_line_gap)),
                    Cell::new(format!("{:.4}", ads.dimgap)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMCEN"),
                    Cell::new(format!("{:.4}", dds.center_mark_size)),
                    Cell::new(format!("{:.4}", ads.dimcen)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMTSZ"),
                    Cell::new(format!("{:.4}", dds.dimensioning_tick_size)),
                    Cell::new(format!("{:.4}", ads.dimtsz)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMLFAC"),
                    Cell::new(format!("{:.4}", dds.dimension_linear_measurement_scale_factor)),
                    Cell::new(format!("{:.4}", ads.dimlfac)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMCLRD"),
                    Cell::new(format!("{:?}", dds.dimension_line_color.index())),
                    Cell::new(format!("{}", ads.dimclrd)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMCLRE"),
                    Cell::new(format!("{:?}", dds.dimension_extension_line_color.index())),
                    Cell::new(format!("{}", ads.dimclre)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMCLRT"),
                    Cell::new(format!("{:?}", dds.dimension_text_color.index())),
                    Cell::new(format!("{}", ads.dimclrt)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMTOL"),
                    Cell::new(format!("{}", dds.generate_dimension_tolerances)),
                    Cell::new(format!("{}", ads.dimtol)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMLIM"),
                    Cell::new(format!("{}", dds.generate_dimension_limits)),
                    Cell::new(format!("{}", ads.dimlim)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMTIH"),
                    Cell::new(format!("{}", dds.dimension_text_inside_horizontal)),
                    Cell::new(format!("{}", ads.dimtih)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMTOH"),
                    Cell::new(format!("{}", dds.dimension_text_outside_horizontal)),
                    Cell::new(format!("{}", ads.dimtoh)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMSE1"),
                    Cell::new(format!("{}", dds.suppress_first_dimension_extension_line)),
                    Cell::new(format!("{}", ads.dimse1)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMSE2"),
                    Cell::new(format!("{}", dds.suppress_second_dimension_extension_line)),
                    Cell::new(format!("{}", ads.dimse2)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMALT"),
                    Cell::new(format!("{}", dds.use_alternate_dimensioning)),
                    Cell::new(format!("{}", ads.dimalt)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMSAH"),
                    Cell::new(format!("{}", dds.use_separate_arrow_blocks_for_dimensions)),
                    Cell::new(format!("{}", ads.dimsah)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMTXSTY"),
                    Cell::new(&dds.dimension_text_style),
                    Cell::new(&ads.dimtxsty),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMDEC"),
                    Cell::new(format!("{}", dds.dimension_precision)),
                    Cell::new(format!("{}", ads.dimdec)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMRND"),
                    Cell::new(format!("{:.4}", dds.dimension_distance_rounding_value)),
                    Cell::new(format!("{:.4}", ads.dimrnd)),
                ]);
                table.add_row(vec![
                    Cell::new("  DIMPOST"),
                    Cell::new(format!("\"{}\"", dds.dimensioning_suffix)),
                    Cell::new(format!("\"{}\"", ads.dimpost)),
                ]);
            }
        }

        // acadrust-only dimstyles
        for ads in acad_doc.dim_styles.iter() {
            if !dxf_drawing.dim_styles().any(|d| d.name == ads.name) {
                table.add_row(vec![
                    Cell::new(format!("[{} (acadrust only)]", ads.name)),
                    Cell::new("—"),
                    Cell::new("✓"),
                ]);
            }
        }

        println!("DimStyle detail:\n{}\n", table);
    }

    // --- 10. Block definitions detail ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Block"),
            Cell::new("dxf-rs entities"),
            Cell::new("acadrust entity_handles"),
            Cell::new("dxf-rs base_point"),
            Cell::new("acadrust base_point (block_record)"),
        ]);

        // dxf-rs blocks
        let dxf_blocks: Vec<_> = dxf_drawing.blocks().collect();
        for db in &dxf_blocks {
            let abr = acad_doc.block_records.iter().find(|br| br.name == db.name);
            let (a_ent, _a_bp) = if let Some(br) = abr {
                (format!("{}", br.entity_handles.len()), format!("—"))
            } else {
                ("—".into(), "—".into())
            };
            table.add_row(vec![
                Cell::new(&db.name),
                Cell::new(format!("{}", db.entities.len())),
                Cell::new(a_ent),
                Cell::new(format!("({:.2}, {:.2}, {:.2})", db.base_point.x, db.base_point.y, db.base_point.z)),
                Cell::new(if abr.is_some() { "✓" } else { "—" }),
            ]);
        }

        // acadrust-only block records
        for abr in acad_doc.block_records.iter() {
            if !dxf_blocks.iter().any(|db| db.name == abr.name) {
                table.add_row(vec![
                    Cell::new(format!("{} (acadrust only)", abr.name)),
                    Cell::new("—"),
                    Cell::new(format!("{}", abr.entity_handles.len())),
                    Cell::new("—"),
                    Cell::new("✓"),
                ]);
            }
        }

        println!("Block definitions:\n{}\n", table);
    }

    // --- 11. Object type breakdown ---
    {
        let mut dxf_obj_types: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for obj in dxf_drawing.objects() {
            let name = match &obj.specific {
                dxf::objects::ObjectType::Dictionary(_) => "Dictionary",
                dxf::objects::ObjectType::DictionaryWithDefault(_) => "DictionaryWithDefault",
                dxf::objects::ObjectType::Layout(_) => "Layout",
                dxf::objects::ObjectType::MLineStyle(_) => "MLineStyle",
                dxf::objects::ObjectType::PlotSettings(_) => "PlotSettings",
                dxf::objects::ObjectType::Material(_) => "Material",
                dxf::objects::ObjectType::VisualStyle(_) => "VisualStyle",
                dxf::objects::ObjectType::TableStyle(_) => "TableStyle",
                dxf::objects::ObjectType::MLeaderStyle(_) => "MLeaderStyle",
                dxf::objects::ObjectType::XRecordObject(_) => "XRecord",
                dxf::objects::ObjectType::PlaceHolder(_) => "PlaceHolder",
                dxf::objects::ObjectType::DictionaryVariable(_) => "DictionaryVariable",
                dxf::objects::ObjectType::ImageDefinition(_) => "ImageDefinition",
                dxf::objects::ObjectType::ImageDefinitionReactor(_) => "ImageDefinitionReactor",
                dxf::objects::ObjectType::Group(_) => "Group",
                dxf::objects::ObjectType::RasterVariables(_) => "RasterVariables",
                dxf::objects::ObjectType::SortentsTable(_) => "SortentsTable",
                dxf::objects::ObjectType::SpatialFilter(_) => "SpatialFilter",
                dxf::objects::ObjectType::GeoData(_) => "GeoData",
                dxf::objects::ObjectType::WipeoutVariables(_) => "WipeoutVariables",
                _ => "Other",
            };
            *dxf_obj_types.entry(name.to_string()).or_insert(0) += 1;
        }

        let mut acad_obj_types: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for (_handle, obj) in &acad_doc.objects {
            let name = match obj {
                acadrust::objects::ObjectType::Dictionary(_) => "Dictionary",
                acadrust::objects::ObjectType::DictionaryWithDefault(_) => "DictionaryWithDefault",
                acadrust::objects::ObjectType::Layout(_) => "Layout",
                acadrust::objects::ObjectType::MLineStyle(_) => "MLineStyle",
                acadrust::objects::ObjectType::PlotSettings(_) => "PlotSettings",
                acadrust::objects::ObjectType::Material(_) => "Material",
                acadrust::objects::ObjectType::VisualStyle(_) => "VisualStyle",
                acadrust::objects::ObjectType::TableStyle(_) => "TableStyle",
                acadrust::objects::ObjectType::MultiLeaderStyle(_) => "MLeaderStyle",
                acadrust::objects::ObjectType::XRecord(_) => "XRecord",
                acadrust::objects::ObjectType::PlaceHolder(_) => "PlaceHolder",
                acadrust::objects::ObjectType::DictionaryVariable(_) => "DictionaryVariable",
                acadrust::objects::ObjectType::ImageDefinition(_) => "ImageDefinition",
                acadrust::objects::ObjectType::ImageDefinitionReactor(_) => "ImageDefinitionReactor",
                acadrust::objects::ObjectType::Group(_) => "Group",
                acadrust::objects::ObjectType::RasterVariables(_) => "RasterVariables",
                acadrust::objects::ObjectType::SortEntitiesTable(_) => "SortentsTable",
                acadrust::objects::ObjectType::SpatialFilter(_) => "SpatialFilter",
                acadrust::objects::ObjectType::GeoData(_) => "GeoData",
                acadrust::objects::ObjectType::WipeoutVariables(_) => "WipeoutVariables",
                acadrust::objects::ObjectType::Scale(_) => "Scale",
                acadrust::objects::ObjectType::BookColor(_) => "BookColor",
                _ => "Other",
            };
            *acad_obj_types.entry(name.to_string()).or_insert(0) += 1;
        }

        let mut all_obj: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for k in dxf_obj_types.keys() { all_obj.insert(k.clone()); }
        for k in acad_obj_types.keys() { all_obj.insert(k.clone()); }

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Object Type"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
            Cell::new("match?"),
        ]);

        for typ in &all_obj {
            let d = dxf_obj_types.get(typ).copied().unwrap_or(0);
            let a = acad_obj_types.get(typ).copied().unwrap_or(0);
            let status = if d == a { "✓" } else { "✗" };
            table.add_row(vec![
                Cell::new(typ),
                Cell::new(d),
                Cell::new(a),
                Cell::new(status),
            ]);
        }

        println!("Object type breakdown:\n{}\n", table);
    }

    // --- 12. ViewPort detail ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("ViewPort"),
            Cell::new("Field"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        for dvp in dxf_drawing.view_ports() {
            let avp = acad_doc.vports.iter().find(|v| v.name == dvp.name);
            table.add_row(vec![
                Cell::new(&dvp.name),
                Cell::new("view_height"),
                Cell::new(format!("{:.4}", dvp.view_height)),
                Cell::new(avp.map(|v| format!("{:.4}", v.view_height)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("view_center"),
                Cell::new(format!("({:.2}, {:.2})", dvp.view_center.x, dvp.view_center.y)),
                Cell::new(avp.map(|v| format!("({:.2}, {:.2})", v.view_center.x, v.view_center.y)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("view_direction"),
                Cell::new(format!("({:.2}, {:.2}, {:.2})", dvp.view_direction.x, dvp.view_direction.y, dvp.view_direction.z)),
                Cell::new(avp.map(|v| format!("({:.2}, {:.2}, {:.2})", v.view_direction.x, v.view_direction.y, v.view_direction.z)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("lens_length"),
                Cell::new(format!("{:.4}", dvp.lens_length)),
                Cell::new(avp.map(|v| format!("{:.4}", v.lens_length)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("aspect_ratio"),
                Cell::new(format!("{:.4}", dvp.view_port_aspect_ratio)),
                Cell::new(avp.map(|v| format!("{:.4}", v.aspect_ratio)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("lower_left"),
                Cell::new(format!("({:.2}, {:.2})", dvp.lower_left.x, dvp.lower_left.y)),
                Cell::new(avp.map(|v| format!("({:.2}, {:.2})", v.lower_left.x, v.lower_left.y)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("upper_right"),
                Cell::new(format!("({:.2}, {:.2})", dvp.upper_right.x, dvp.upper_right.y)),
                Cell::new(avp.map(|v| format!("({:.2}, {:.2})", v.upper_right.x, v.upper_right.y)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("grid_spacing"),
                Cell::new(format!("({:.2}, {:.2})", dvp.grid_spacing.x, dvp.grid_spacing.y)),
                Cell::new(avp.map(|v| format!("({:.2}, {:.2})", v.grid_spacing.x, v.grid_spacing.y)).unwrap_or("—".into())),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("snap_spacing"),
                Cell::new(format!("({:.2}, {:.2})", dvp.snap_spacing.x, dvp.snap_spacing.y)),
                Cell::new(avp.map(|v| format!("({:.2}, {:.2})", v.snap_spacing.x, v.snap_spacing.y)).unwrap_or("—".into())),
            ]);
        }

        println!("ViewPort detail:\n{}\n", table);
    }

    // --- 13. AppId detail ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("AppId"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        let mut dxf_appids: Vec<_> = dxf_drawing.app_ids().map(|a| a.name.clone()).collect();
        dxf_appids.sort();
        let acad_appids: Vec<_> = acad_doc.app_ids.iter().map(|a| a.name.clone()).collect();

        let mut all_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for n in &dxf_appids { all_names.insert(n.clone()); }
        for n in &acad_appids { all_names.insert(n.clone()); }

        for name in &all_names {
            let in_dxf = if dxf_appids.contains(name) { "✓" } else { "—" };
            let in_acad = if acad_appids.contains(name) { "✓" } else { "—" };
            table.add_row(vec![
                Cell::new(name),
                Cell::new(in_dxf),
                Cell::new(in_acad),
            ]);
        }

        println!("AppIds:\n{}\n", table);
    }

    // --- 14. Text / MText content spot-check ---
    {
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Text #"),
            Cell::new("Field"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        let dxf_texts: Vec<_> = dxf_drawing.entities().filter_map(|e| match &e.specific {
            dxf::entities::EntityType::Text(t) => Some((t.clone(), e.common.layer.clone())),
            _ => None,
        }).take(3).collect();

        let acad_texts: Vec<_> = acad_doc.entities().filter_map(|e| match e {
            acadrust::entities::EntityType::Text(t) => Some(t),
            _ => None,
        }).take(3).collect();

        for (i, ((dt, dl), at)) in dxf_texts.iter().zip(acad_texts.iter()).enumerate() {
            let label = format!("TEXT #{}", i + 1);
            table.add_row(vec![
                Cell::new(&label),
                Cell::new("value"),
                Cell::new(format!("\"{}\"", dt.value)),
                Cell::new(format!("\"{}\"", at.value)),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("location"),
                Cell::new(format!("({:.2}, {:.2}, {:.2})", dt.location.x, dt.location.y, dt.location.z)),
                Cell::new(format!("({:.2}, {:.2}, {:.2})", at.insertion_point.x, at.insertion_point.y, at.insertion_point.z)),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("height"),
                Cell::new(format!("{:.4}", dt.text_height)),
                Cell::new(format!("{:.4}", at.height)),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("rotation"),
                Cell::new(format!("{:.4}", dt.rotation)),
                Cell::new(format!("{:.4}", at.rotation)),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("style"),
                Cell::new(&dt.text_style_name),
                Cell::new(&at.style),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("layer"),
                Cell::new(&dl),
                Cell::new(&at.common.layer),
            ]);
        }

        // MText spot-check
        let dxf_mtexts: Vec<_> = dxf_drawing.entities().filter_map(|e| match &e.specific {
            dxf::entities::EntityType::MText(t) => Some((t.clone(), e.common.layer.clone())),
            _ => None,
        }).take(3).collect();

        let acad_mtexts: Vec<_> = acad_doc.entities().filter_map(|e| match e {
            acadrust::entities::EntityType::MText(t) => Some(t),
            _ => None,
        }).take(3).collect();

        for (i, ((dm, dl), am)) in dxf_mtexts.iter().zip(acad_mtexts.iter()).enumerate() {
            let label = format!("MTEXT #{}", i + 1);
            let dxf_text = if dm.text.is_empty() {
                dm.extended_text.join("")
            } else {
                dm.text.clone()
            };
            table.add_row(vec![
                Cell::new(&label),
                Cell::new("value"),
                Cell::new(format!("\"{}\"", &dxf_text[..dxf_text.len().min(40)])),
                Cell::new(format!("\"{}\"", &am.value[..am.value.len().min(40)])),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("insertion_point"),
                Cell::new(format!("({:.2}, {:.2}, {:.2})", dm.insertion_point.x, dm.insertion_point.y, dm.insertion_point.z)),
                Cell::new(format!("({:.2}, {:.2}, {:.2})", am.insertion_point.x, am.insertion_point.y, am.insertion_point.z)),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("height"),
                Cell::new(format!("{:.4}", dm.initial_text_height)),
                Cell::new(format!("{:.4}", am.height)),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("rotation"),
                Cell::new(format!("{:.4}", dm.rotation_angle)),
                Cell::new(format!("{:.4}", am.rotation)),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("style"),
                Cell::new(&dm.text_style_name),
                Cell::new(&am.style),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("layer"),
                Cell::new(&dl),
                Cell::new(&am.common.layer),
            ]);
        }

        if !dxf_texts.is_empty() || !dxf_mtexts.is_empty() {
            println!("Text / MText spot-check (first 3 of each):\n{}\n", table);
        }
    }

    // --- 15. Handle distribution summary ---
    {
        let mut dxf_handles: Vec<u64> = Vec::new();
        for e in dxf_drawing.entities() {
            if e.common.handle.0 != 0 {
                dxf_handles.push(e.common.handle.0);
            }
        }

        let mut acad_handles: Vec<u64> = Vec::new();
        for e in acad_doc.entities() {
            let h = e.common().handle;
            if !h.is_null() {
                acad_handles.push(h.into());
            }
        }

        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Handle Metric"),
            Cell::new("dxf-rs"),
            Cell::new("acadrust"),
        ]);

        table.add_row(vec![
            Cell::new("Entity handles count"),
            Cell::new(dxf_handles.len()),
            Cell::new(acad_handles.len()),
        ]);

        if !dxf_handles.is_empty() {
            let dmin = dxf_handles.iter().min().unwrap();
            let dmax = dxf_handles.iter().max().unwrap();
            table.add_row(vec![
                Cell::new("Entity handle range"),
                Cell::new(format!("0x{:X}..0x{:X}", dmin, dmax)),
                Cell::new(if !acad_handles.is_empty() {
                    let amin = acad_handles.iter().min().unwrap();
                    let amax = acad_handles.iter().max().unwrap();
                    format!("0x{:X}..0x{:X}", amin, amax)
                } else {
                    "—".into()
                }),
            ]);
        }

        // Table control handles
        let mut dxf_table_handles = 0usize;
        for l in dxf_drawing.layers() { if l.handle.0 != 0 { dxf_table_handles += 1; } }
        for lt in dxf_drawing.line_types() { if lt.handle.0 != 0 { dxf_table_handles += 1; } }
        for s in dxf_drawing.styles() { if s.handle.0 != 0 { dxf_table_handles += 1; } }
        for ds in dxf_drawing.dim_styles() { if ds.handle.0 != 0 { dxf_table_handles += 1; } }
        for a in dxf_drawing.app_ids() { if a.handle.0 != 0 { dxf_table_handles += 1; } }

        let acad_table_handles = acad_doc.layers.len()
            + acad_doc.line_types.len()
            + acad_doc.text_styles.len()
            + acad_doc.dim_styles.len()
            + acad_doc.app_ids.len()
            + acad_doc.block_records.len()
            + acad_doc.views.len()
            + acad_doc.vports.len()
            + acad_doc.ucss.len();

        table.add_row(vec![
            Cell::new("Table entry handles"),
            Cell::new(dxf_table_handles),
            Cell::new(acad_table_handles),
        ]);

        table.add_row(vec![
            Cell::new("Object handles"),
            Cell::new(dxf_drawing.objects().count()),
            Cell::new(acad_doc.objects.len()),
        ]);

        println!("Handle distribution:\n{}\n", table);
    }
}
