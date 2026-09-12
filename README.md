# acadrust-dxf-bench

Benchmark suite comparing four DXF/DWG libraries across parsing, writing, roundtrip, binary DXF, and DWG operations.

| Library | Language | DXF (ASCII) | DXF (Binary) | DWG |
|---|---|---|---|---|
| **[dxf-rs](https://crates.io/crates/dxf)** v0.6 | Rust | Read/Write | Read/Write | — |
| **[acadrust](https://crates.io/crates/acadrust)** v0.3.0 | Rust | Read/Write | Read/Write | Read/Write |
| **[ACadSharp](https://github.com/DomCR/ACadSharp)** v3.4.9 | C# (.NET) | Read/Write | Read/Write | Read/Write |
| **[ezdxf](https://github.com/mozman/ezdxf)** v1.4.3 | Python | Read/Write | Read/Write | — |

## What's Measured

| Category | Scenarios |
|---|---|
| **DXF Parse** | 7 entity-type variants (lines, circles, arcs, ellipses, mixed, polylines, 3D) |
| **DXF Write** | Lines-only and mixed entities |
| **DXF Roundtrip** | Read → write (mixed) |
| **Binary DXF Parse** | Binary mixed and binary lines |
| **Binary DXF Write** | Binary lines and binary mixed |
| **Binary DXF Roundtrip** | Binary mixed roundtrip |
| **DWG Parse** | DWG mixed and DWG lines |
| **DWG Write** | DWG lines and DWG mixed (acadrust + ACadSharp only) |
| **DWG Roundtrip** | DWG mixed roundtrip (acadrust + ACadSharp only) |

### Scale Presets

| Preset | Entity Count |
|---|---|
| `small` | 100 |
| `medium` | 1,000 |
| `large` | 10,000 |
| `huge` | 100,000 |

## Quick Start

```bash
# Native GUI: choose a preset or add your own DXF/DXB/DWG files
cargo run --release -- --gui

# Quick comparison table (CLI runner)
cargo run --release -- --scale large --iterations 10

# Benchmark user files (repeat --input for multiple files)
cargo run --release -- --input drawing.dxf --input model.dwg --iterations 10

# Full Criterion benchmarks
cargo bench

# Individual benchmark suites
cargo bench --bench parse_bench
cargo bench --bench write_bench
cargo bench --bench roundtrip_bench
```

The GUI has a file picker for `.dxf`, binary `.dxb`, and `.dwg` files. It runs
the selected preset when no custom files are selected, or benchmarks each
selected file when custom files are present. Each run automatically writes a
Markdown report and a JSON sidecar. Preset reports are written below
`bench_output/<scale>/report.md`; custom reports are written below
`bench_output/custom/report.md`.

The CLI accepts the same custom-file inputs and also supports
`--report path/to/report.md` to choose the Markdown report location. The JSON
report is written next to it with the same filename and a `.json` extension.

### Windows Package

Build a portable GUI package with the ACadSharp and ezdxf runners embedded as
adjacent executables:

```powershell
.\scripts\package-windows.ps1
```

The package is written to `dist/acadrust-dxf-bench/`. Keep
`acadrust-dxf-bench.exe`, `acadsharp-bench.exe`, and `ezdxf-bench.exe` together.
The GUI prefers those bundled runners and does not require a system Python,
`ezdxf` installation, or .NET SDK.

## Architecture

The Rust binary (main.rs) orchestrates all benchmarks:

- **dxf-rs** and **acadrust** are timed directly in-process via Rust APIs
- **ACadSharp** runs as a .NET subprocess (`acadsharp-bench/`) outputting JSON
- **ezdxf** runs as a Python subprocess (`ezdxf-bench/`) outputting JSON

All libraries operate on the same generated test files under `bench_output/<scale>/`.

## Project Structure

```
├── src/
│   ├── main.rs           # CLI benchmark harness (orchestrates all 4 libraries)
│   ├── lib.rs            # Crate root
│   └── generators.rs     # DXF/DWG/DXB test-data generators
├── benches/              # Criterion benchmarks (parse, write, roundtrip)
├── acadsharp-bench/      # C# (.NET) ACadSharp benchmark runner
├── ezdxf-bench/          # Python ezdxf benchmark runner
├── reports/              # Generated benchmark reports (Markdown)
└── bench_output/         # Generated test files and results
```

## Requirements

- Rust 1.70+
- .NET 8.0 SDK (for ACadSharp benchmarks)
- Python 3.10+ with `ezdxf` installed (for ezdxf benchmarks)
- ~500 MB free RAM for `huge` scale
