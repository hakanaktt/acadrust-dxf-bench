# acadrust-dxf-bench

Heavy-duty DXF parsing/writing performance benchmark suite comparing **[dxf-rs](https://crates.io/crates/dxf)** (v0.6) and **[acadrust](https://crates.io/crates/acadrust)** (v0.2).

## What's Measured

### Parse Benchmarks (`benches/parse_bench.rs`)
| Scenario | Description |
|---|---|
| **By entity type** | Lines, circles, arcs, ellipses, mixed, polylines, 3D entities — at 4 scale levels |
| **File I/O** | Parse from disk (10k mixed entities) |
| **Scaling** | Lines only at 100 → 50,000 entities to measure scaling behavior |

### Write Benchmarks (`benches/write_bench.rs`)
| Scenario | Description |
|---|---|
| **To memory** | Lines-only and mixed entities at 4 scale levels |
| **To file** | Write to disk (10k lines & mixed) |
| **Scaling** | Lines only at 100 → 50,000 entities |

### Roundtrip Benchmarks (`benches/roundtrip_bench.rs`)
| Scenario | Description |
|---|---|
| **Same-library** | Parse → write within each library |
| **Cross-library** | Full roundtrip baselines at 10k mixed |

### Scale Presets

| Preset | Entity Count |
|---|---|
| `small` | 100 |
| `medium` | 1,000 |
| `large` | 10,000 |
| `huge` | 100,000 |

## Quick Start

### Quick comparison table (CLI runner)

```bash
cargo run --release -- --scale large --iterations 10
```

### Full Criterion benchmarks

```bash
# Run all benchmarks
cargo bench

# Run only parse benchmarks
cargo bench --bench parse_bench

# Run only write benchmarks
cargo bench --bench write_bench

# Run only roundtrip benchmarks
cargo bench --bench roundtrip_bench

# Filter specific benchmark
cargo bench --bench parse_bench -- "parse/large_10k"
```

### HTML Reports

After running `cargo bench`, open `target/criterion/report/index.html` for detailed
statistical reports with violin plots and regression analysis.

## Project Structure

```
├── Cargo.toml
├── src/
│   ├── lib.rs            # Crate root
│   ├── main.rs           # CLI quick-comparison runner
│   └── generators.rs     # DXF test-data generators (all scales & entity types)
├── benches/
│   ├── parse_bench.rs    # Criterion parse benchmarks
│   ├── write_bench.rs    # Criterion write benchmarks
│   └── roundtrip_bench.rs# Criterion roundtrip benchmarks
└── README.md
```

## Example Output (CLI)

```
=== DXF Benchmark: dxf-rs vs acadrust  (scale=large, entities=10000, iterations=10) ===

Test files generated:
  lines_only              1234567 bytes
  circles_only             987654 bytes
  ...

┌────────────────┬─────────────┬────────────────┬──────────────────┬──────────┐
│ PARSE          │ dxf-rs (ms) │ acadrust (ms)  │ ratio (dxf/acad) │ faster   │
├────────────────┼─────────────┼────────────────┼──────────────────┼──────────┤
│ lines_only     │ 45.23       │ 32.10          │ 1.41x            │ acadrust │
│ circles_only   │ 42.11       │ 30.88          │ 1.36x            │ acadrust │
│ ...            │ ...         │ ...            │ ...              │ ...      │
└────────────────┴─────────────┴────────────────┴──────────────────┴──────────┘
```

## Requirements

- Rust 1.70+
- ~500 MB free RAM for `huge` scale benchmarks

## License

MIT
