# Binary DXF (DXB) Parse Performance Report

**Date:** March 12, 2026  
**Libraries:** dxf-rs v0.6, acadrust v0.3.0, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)  
**Test data:** Binary DXF files generated from mixed and lines-only workloads

---

## Summary

For binary DXF parsing, **dxf-rs and acadrust are closely matched**, trading wins depending on scale and workload. **ACadSharp** is significantly slower — 4–5× behind the Rust libraries.

| Scale | binary_mixed Winner | binary_lines Winner |
|---|---|---|
| Small (100) | dxf-rs (1.03×) | acadrust (1.09×) |
| Medium (1K) | dxf-rs (1.06×) | dxf-rs (1.35×) |
| Large (10K) | acadrust (1.05×) | dxf-rs (1.12×) |
| Huge (100K) | dxf-rs (1.01×) | acadrust (1.21×) |

---

## Detailed Results

All times in **milliseconds** (lower is better).

### Small (100 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| binary_mixed | **0.38** | 0.39 | 3.50 | dxf-rs |
| binary_lines | 0.38 | **0.35** | 2.79 | acadrust |

### Medium (1,000 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| binary_mixed | **2.08** | 2.21 | 16.44 | dxf-rs |
| binary_lines | **1.75** | 2.36 | 18.85 | dxf-rs |

### Large (10,000 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| binary_mixed | 25.37 | **24.23** | 123.51 | acadrust |
| binary_lines | **20.45** | 18.22 | 99.33 | acadrust |

### Huge (100,000 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| binary_mixed | **214.20** | 215.65 | 1058.37 | dxf-rs |
| binary_lines | 231.19 | **191.28** | 789.54 | acadrust |

---

## Observations

- Binary DXF parsing is a much tighter race between dxf-rs and acadrust than ASCII DXF parsing. Neither library has a clear advantage.
- **ACadSharp** parses binary DXF 4–8× slower than the Rust libraries, likely because `DxfReader` in ACadSharp uses the same ASCII-oriented code path for binary format detection.
- Binary DXF files are ~55–60% the size of their ASCII equivalents, yet parse times don't shrink proportionally — binary format still requires header/entity structure parsing.
