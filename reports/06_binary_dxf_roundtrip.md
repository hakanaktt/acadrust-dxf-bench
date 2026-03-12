# Binary DXF (DXB) Roundtrip Performance Report

**Date:** March 12, 2026  
**Libraries:** dxf-rs v0.6, acadrust v0.3.0, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)  
**Test data:** Binary DXF mixed files, read from disk and written back as binary DXF

---

## Summary

**acadrust is faster for binary DXF roundtrip** at every scale, with its write-side advantage compensating for the near-even parse performance. ACadSharp participates only in the read half (no binary write), so its roundtrip involves reading binary DXF then writing ASCII DXF.

| Scale | dxf-rs (ms) | acadrust (ms) | ACadSharp (ms) | Speedup (acadrust vs dxf-rs) |
|---|---|---|---|---|
| Small (100) | 1.45 | **0.72** | 8.25 | 2.01× |
| Medium (1K) | 6.55 | **4.18** | 34.19 | 1.57× |
| Large (10K) | 35.72 | **29.57** | 156.73 | 1.21× |
| Huge (100K) | 329.45 | **207.27** | 1489.78 | 1.59× |

---

## Observations

- acadrust's binary roundtrip advantage comes primarily from its binary write performance (up to 9× faster than dxf-rs).
- **ACadSharp** roundtrip is 5–7× slower than acadrust because it reads binary DXF then writes ASCII DXF (binary write not supported).
- All binary roundtrip output files are preserved on disk for validation under `bench_output/<scale>/roundtrip/`.
- At huge scale, acadrust roundtrips 11.5 MB of binary DXF in ~207 ms.
