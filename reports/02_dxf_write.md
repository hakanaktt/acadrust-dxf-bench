# DXF Write Performance Report

**Date:** March 12, 2026  
**Libraries:** dxf-rs v0.6, acadrust v0.3.0, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)  
**Test data:** In-memory documents written to disk files

---

## Summary

**acadrust is the fastest DXF writer** at small and large+ scales. At **medium** scale, dxf-rs has a slight edge. ACadSharp is consistently the slowest, 2–3× behind the Rust libraries.

| Scale | Lines Winner | Mixed Winner |
|---|---|---|
| Small (100) | acadrust (1.92×) | dxf-rs (1.59×) |
| Medium (1K) | dxf-rs (1.08×) | dxf-rs (1.09×) |
| Large (10K) | acadrust (1.18×) | acadrust (1.35×) |
| Huge (100K) | acadrust (1.22×) | acadrust (1.34×) |

---

## Detailed Results

All times in **milliseconds** (lower is better).

### Small (100 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| lines_only | 1.52 | **0.79** | 4.53 | acadrust |
| mixed | **1.33** | 2.12 | 4.48 | dxf-rs |

### Medium (1,000 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| lines_only | **5.75** | 6.23 | 10.33 | dxf-rs |
| mixed | **4.89** | 5.32 | 10.76 | dxf-rs |

### Large (10,000 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| lines_only | 30.65 | **25.87** | 105.78 | acadrust |
| mixed | 38.30 | **28.46** | 51.93 | acadrust |

### Huge (100,000 entities)

| Workload | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Fastest |
|---|---|---|---|---|
| lines_only | 304.30 | **248.62** | 555.82 | acadrust |
| mixed | 330.45 | **246.07** | 620.75 | acadrust |

---

## Observations

- **acadrust** pulls ahead at large scale where its optimized serialization pays off.
- **dxf-rs** has marginally better write performance at medium scale — possibly benefiting from simpler internal structures at that entity count.
- **ACadSharp** is ~2–3× slower than both Rust libraries for writing, reflecting the overhead of .NET object model serialization.
- **Mixed entities** are more expensive to write than lines-only at huge scale due to more complex group code patterns.
