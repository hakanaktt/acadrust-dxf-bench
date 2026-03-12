# DXF Roundtrip Performance Report

**Date:** March 12, 2026  
**Libraries:** dxf-rs v0.6, acadrust v0.3.0, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)  
**Test data:** Mixed-entity DXF files, read from disk and written back to disk

---

## Summary

**acadrust is the fastest for DXF roundtrip** (parse + write) at every scale, roughly 1.7–2× faster than dxf-rs and 3–4× faster than ACadSharp.

| Scale | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Speedup (acadrust vs dxf-rs) |
|---|---|---|---|---|
| Small (100) | 4.12 | **1.42** | 9.04 | 2.90× |
| Medium (1K) | 11.82 | **5.95** | 23.60 | 1.99× |
| Large (10K) | 90.72 | **49.93** | 170.07 | 1.82× |
| Huge (100K) | 914.07 | **477.18** | 2028.86 | 1.92× |

---

## Observations

- Roundtrip performance is the sum of parse + write, so acadrust's advantages in both areas compound.
- **ACadSharp's** roundtrip is about 4× slower than acadrust at huge scale — the combined effect of slower parsing and slower writing in a managed runtime.
- All roundtrip output files are preserved on disk under `bench_output/<scale>/roundtrip/` for validation.
- At huge scale (100K entities), acadrust completes a full roundtrip in under 500 ms while ACadSharp needs over 2 seconds.
