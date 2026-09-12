use crate::python;
use crate::report::{write_report, ReportSection, TimingResult};
use crate::runners;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct TimingEntry {
    #[serde(rename = "Label")]
    label: String,
    #[serde(rename = "Ms")]
    ms: f64,
}

type ExternalResults = HashMap<String, Vec<TimingEntry>>;

/// Benchmark user-provided DXF, binary DXF, or DWG files.
pub fn run(
    paths: &[PathBuf],
    iterations: usize,
    report_path: Option<PathBuf>,
) -> Result<(), String> {
    if iterations == 0 {
        return Err("iterations must be greater than zero".into());
    }
    if paths.is_empty() {
        return Err("at least one input file is required".into());
    }

    let report_path = report_path.unwrap_or_else(|| PathBuf::from("bench_output/custom/report.md"));
    let run_dir = report_path
        .parent()
        .unwrap_or_else(|| Path::new("bench_output/custom"))
        .join("files");
    fs::create_dir_all(&run_dir).map_err(|e| format!("create custom output directory: {e}"))?;

    let mut parse_results = Vec::new();
    let mut write_results = Vec::new();
    let mut roundtrip_results = Vec::new();

    for (index, source) in paths.iter().enumerate() {
        let source = source
            .canonicalize()
            .map_err(|e| format!("{}: {e}", source.display()))?;
        if !source.is_file() {
            return Err(format!("input is not a file: {}", source.display()));
        }

        let kind = FileKind::from_path(&source)?;
        let label = format!("{} ({})", source.display(), kind.label());
        let staging = run_dir.join(format!("{:02}_{}", index + 1, safe_stem(&source)));
        fs::create_dir_all(&staging).map_err(|e| format!("create staging directory: {e}"))?;
        let staged_input = stage_input(&source, &staging, kind)?;

        println!("\n=== Custom benchmark: {} ===", source.display());
        println!(
            "Format: {}, size: {} bytes",
            kind.label(),
            fs::metadata(&source).map(|m| m.len()).unwrap_or(0)
        );

        let (acadsharp, ezdxf) = run_external(&staging, iterations);
        let (dxf_ms, acadrust_ms) = time_parse(&source, kind, iterations);
        parse_results.push(TimingResult {
            label: label.clone(),
            dxf_ms,
            acadrust_ms,
            acadsharp_ms: external_value(&acadsharp, kind.parse_category(), kind.external_label()),
            ezdxf_ms: external_value(&ezdxf, kind.parse_category(), kind.external_label()),
        });

        let (dxf_write, acadrust_write) = time_write(&source, kind, &staging, iterations);
        write_results.push(TimingResult {
            label: label.clone(),
            dxf_ms: dxf_write,
            acadrust_ms: acadrust_write,
            acadsharp_ms: external_value(&acadsharp, kind.write_category(), kind.external_label()),
            ezdxf_ms: external_value(&ezdxf, kind.write_category(), kind.external_label()),
        });

        let (dxf_rt, acadrust_rt) = time_roundtrip(&source, kind, &staging, iterations);
        roundtrip_results.push(TimingResult {
            label,
            dxf_ms: dxf_rt,
            acadrust_ms: acadrust_rt,
            acadsharp_ms: external_value(
                &acadsharp,
                kind.roundtrip_category(),
                kind.roundtrip_label(),
            ),
            ezdxf_ms: external_value(&ezdxf, kind.roundtrip_category(), kind.roundtrip_label()),
        });
        let _ = staged_input;
    }

    let sections = vec![
        ReportSection::new("Parse", parse_results.clone()),
        ReportSection::new("Write", write_results.clone()),
        ReportSection::new("Roundtrip", roundtrip_results.clone()),
    ];

    print_custom_table("CUSTOM PARSE", &parse_results);
    print_custom_table("CUSTOM WRITE", &write_results);
    print_custom_table("CUSTOM ROUNDTRIP", &roundtrip_results);

    let (markdown, json) = write_report(&report_path, "custom files", iterations, &sections)
        .map_err(|e| format!("write report: {e}"))?;
    println!(
        "\nReports written to:\n  {}\n  {}",
        markdown.display(),
        json.display()
    );
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum FileKind {
    Dxf,
    Dxb,
    Dwg,
}

impl FileKind {
    fn from_path(path: &Path) -> Result<Self, String> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "dxf" => Ok(Self::Dxf),
            "dxb" => Ok(Self::Dxb),
            "dwg" => Ok(Self::Dwg),
            _ => Err(format!(
                "unsupported input extension for {}; use .dxf, .dxb, or .dwg",
                path.display()
            )),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Dxf => "DXF",
            Self::Dxb => "binary DXF",
            Self::Dwg => "DWG",
        }
    }

    fn canonical_name(self) -> &'static str {
        match self {
            Self::Dxf => "mixed.dxf",
            Self::Dxb => "binary_mixed.dxb",
            Self::Dwg => "mixed.dwg",
        }
    }

    fn external_label(self) -> &'static str {
        match self {
            Self::Dxf => "mixed",
            Self::Dxb => "binary_mixed",
            Self::Dwg => "dwg_mixed",
        }
    }

    fn parse_category(self) -> &'static str {
        match self {
            Self::Dxf => "parse",
            Self::Dxb => "binary_parse",
            Self::Dwg => "dwg_parse",
        }
    }

    fn write_category(self) -> &'static str {
        match self {
            Self::Dxf => "write",
            Self::Dxb => "binary_write",
            Self::Dwg => "dwg_write",
        }
    }

    fn roundtrip_category(self) -> &'static str {
        match self {
            Self::Dxf => "roundtrip",
            Self::Dxb => "binary_roundtrip",
            Self::Dwg => "dwg_roundtrip",
        }
    }

    fn roundtrip_label(self) -> &'static str {
        match self {
            Self::Dxf => "mixed_roundtrip",
            Self::Dxb => "binary_mixed_roundtrip",
            Self::Dwg => "dwg_mixed_roundtrip",
        }
    }
}

fn safe_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("input")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn stage_input(source: &Path, staging: &Path, kind: FileKind) -> Result<PathBuf, String> {
    let destination = staging.join(kind.canonical_name());
    fs::copy(source, &destination).map_err(|e| format!("stage {}: {e}", source.display()))?;
    Ok(destination)
}

fn external_value(results: &Option<ExternalResults>, category: &str, label: &str) -> f64 {
    results
        .as_ref()
        .and_then(|r| r.get(category))
        .and_then(|entries| entries.iter().find(|entry| entry.label == label))
        .map(|entry| entry.ms)
        .unwrap_or(f64::NAN)
}

fn run_external(
    dir: &Path,
    iterations: usize,
) -> (Option<ExternalResults>, Option<ExternalResults>) {
    let absolute = match dir.canonicalize() {
        Ok(path) => path,
        Err(_) => return (None, None),
    };
    let acadsharp = runners::run_acadsharp(&absolute, iterations)
        .ok()
        .and_then(|output| parse_external_output(&output.stdout, output.status.success()));

    let ezdxf = run_ezdxf(&absolute, iterations);

    (acadsharp, ezdxf)
}

fn run_ezdxf(dir: &Path, iterations: usize) -> Option<ExternalResults> {
    let script = Path::new("ezdxf-bench/bench.py");
    if let Ok(output) = python::run_ezdxf_bench(script, dir, iterations) {
        if let Some(results) = parse_external_output(&output.stdout, output.status.success()) {
            return Some(results);
        }
    }

    None
}

fn parse_external_output(stdout: &[u8], success: bool) -> Option<ExternalResults> {
    if !success {
        return None;
    }
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| line.starts_with('{'))
        .last()
        .and_then(|line| serde_json::from_str(line).ok())
}

fn time_operation<F>(iterations: usize, mut operation: F) -> f64
where
    F: FnMut() -> Result<(), String>,
{
    if operation().is_err() {
        return f64::NAN;
    }
    let start = Instant::now();
    for _ in 0..iterations {
        if operation().is_err() {
            return f64::NAN;
        }
    }
    start.elapsed().as_secs_f64() * 1000.0 / iterations as f64
}

fn time_parse(path: &Path, kind: FileKind, iterations: usize) -> (f64, f64) {
    let dxf_ms = if matches!(kind, FileKind::Dxf | FileKind::Dxb) {
        time_operation(iterations, || {
            let file = fs::File::open(path).map_err(|e| e.to_string())?;
            let mut reader = BufReader::new(file);
            dxf::Drawing::load(&mut reader)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    } else {
        f64::NAN
    };
    let acadrust_ms = time_operation(iterations, || {
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        match kind {
            FileKind::Dxf | FileKind::Dxb => acadrust::DxfReader::from_reader(reader)
                .map_err(|e| e.to_string())?
                .read()
                .map(|_| ())
                .map_err(|e| e.to_string()),
            FileKind::Dwg => acadrust::DwgReader::from_stream(reader)
                .read()
                .map(|_| ())
                .map_err(|e| e.to_string()),
        }
    });
    (dxf_ms, acadrust_ms)
}

fn time_write(path: &Path, kind: FileKind, output_dir: &Path, iterations: usize) -> (f64, f64) {
    match kind {
        FileKind::Dxf | FileKind::Dxb => {
            let dxf_drawing =
                dxf::Drawing::load(&mut BufReader::new(fs::File::open(path).unwrap())).ok();
            let acad_doc = acadrust::DxfReader::from_file(path.to_str().unwrap())
                .ok()
                .and_then(|r| r.read().ok());
            let dxf_ms = dxf_drawing
                .as_ref()
                .map(|drawing| {
                    time_operation(iterations, || {
                        let file = fs::File::create(output_dir.join("write_dxfrs").with_extension(
                            match kind {
                                FileKind::Dxf => "dxf",
                                FileKind::Dxb => "dxb",
                                _ => unreachable!(),
                            },
                        ))
                        .map_err(|e| e.to_string())?;
                        let mut writer = BufWriter::new(file);
                        if matches!(kind, FileKind::Dxf) {
                            drawing.save(&mut writer).map_err(|e| e.to_string())
                        } else {
                            drawing.save_binary(&mut writer).map_err(|e| e.to_string())
                        }
                    })
                })
                .unwrap_or(f64::NAN);
            let acad_ms = acad_doc
                .as_ref()
                .map(|doc| {
                    time_operation(iterations, || {
                        let file =
                            fs::File::create(output_dir.join("write_acadrust").with_extension(
                                match kind {
                                    FileKind::Dxf => "dxf",
                                    FileKind::Dxb => "dxb",
                                    _ => unreachable!(),
                                },
                            ))
                            .map_err(|e| e.to_string())?;
                        let writer = BufWriter::new(file);
                        if matches!(kind, FileKind::Dxf) {
                            acadrust::DxfWriter::new(doc)
                                .write_to_writer(writer)
                                .map_err(|e| e.to_string())
                        } else {
                            acadrust::DxfWriter::new_binary(doc)
                                .write_to_writer(writer)
                                .map_err(|e| e.to_string())
                        }
                    })
                })
                .unwrap_or(f64::NAN);
            (dxf_ms, acad_ms)
        }
        FileKind::Dwg => {
            let acad_doc = fs::File::open(path).ok().and_then(|file| {
                acadrust::DwgReader::from_stream(BufReader::new(file))
                    .read()
                    .ok()
            });
            let acad_ms = acad_doc
                .as_ref()
                .map(|doc| {
                    time_operation(iterations, || {
                        let file = fs::File::create(output_dir.join("write_acadrust.dwg"))
                            .map_err(|e| e.to_string())?;
                        acadrust::DwgWriter::write_to_writer(BufWriter::new(file), doc)
                            .map_err(|e| e.to_string())
                    })
                })
                .unwrap_or(f64::NAN);
            (f64::NAN, acad_ms)
        }
    }
}

fn time_roundtrip(path: &Path, kind: FileKind, output_dir: &Path, iterations: usize) -> (f64, f64) {
    let dxf_ms = if matches!(kind, FileKind::Dxf | FileKind::Dxb) {
        time_operation(iterations, || {
            let mut reader = BufReader::new(fs::File::open(path).map_err(|e| e.to_string())?);
            let drawing = dxf::Drawing::load(&mut reader).map_err(|e| e.to_string())?;
            let file = fs::File::create(output_dir.join("roundtrip_dxfrs").with_extension(
                if matches!(kind, FileKind::Dxf) {
                    "dxf"
                } else {
                    "dxb"
                },
            ))
            .map_err(|e| e.to_string())?;
            let mut writer = BufWriter::new(file);
            if matches!(kind, FileKind::Dxf) {
                drawing.save(&mut writer).map_err(|e| e.to_string())
            } else {
                drawing.save_binary(&mut writer).map_err(|e| e.to_string())
            }
        })
    } else {
        f64::NAN
    };
    let acadrust_ms = time_operation(iterations, || {
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        match kind {
            FileKind::Dxf | FileKind::Dxb => {
                let doc = acadrust::DxfReader::from_reader(reader)
                    .map_err(|e| e.to_string())?
                    .read()
                    .map_err(|e| e.to_string())?;
                let output =
                    fs::File::create(output_dir.join("roundtrip_acadrust").with_extension(
                        if matches!(kind, FileKind::Dxf) {
                            "dxf"
                        } else {
                            "dxb"
                        },
                    ))
                    .map_err(|e| e.to_string())?;
                if matches!(kind, FileKind::Dxf) {
                    acadrust::DxfWriter::new(&doc)
                        .write_to_writer(BufWriter::new(output))
                        .map_err(|e| e.to_string())
                } else {
                    acadrust::DxfWriter::new_binary(&doc)
                        .write_to_writer(BufWriter::new(output))
                        .map_err(|e| e.to_string())
                }
            }
            FileKind::Dwg => {
                let doc = acadrust::DwgReader::from_stream(reader)
                    .read()
                    .map_err(|e| e.to_string())?;
                let output = fs::File::create(output_dir.join("roundtrip_acadrust.dwg"))
                    .map_err(|e| e.to_string())?;
                acadrust::DwgWriter::write_to_writer(BufWriter::new(output), &doc)
                    .map_err(|e| e.to_string())
            }
        }
    });
    (dxf_ms, acadrust_ms)
}

fn print_custom_table(title: &str, results: &[TimingResult]) {
    println!("\n{title}");
    println!("| Input | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | ezdxf (ms) |");
    println!("|---|---:|---:|---:|---:|");
    for result in results {
        let fmt = |value: f64| {
            if value.is_finite() {
                format!("{value:.2}")
            } else {
                "n/a".into()
            }
        };
        println!(
            "| {} | {} | {} | {} | {} |",
            result.label,
            fmt(result.dxf_ms),
            fmt(result.acadrust_ms),
            fmt(result.acadsharp_ms),
            fmt(result.ezdxf_ms)
        );
    }
}
