# DXF Roundtrip Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest for DXF roundtrip** (read → write) at all scales, ~2× faster than dxf-rs, ~3.5× faster than ACadSharp, and ~24× faster than ezdxf at huge scale.

---

## Results (ms, lower is better)

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 1.73 | **1.09** | 7.44 | 17.86 | acadrust |
| Medium | 1,000 | 11.72 | **7.58** | 26.73 | 160.04 | acadrust |
| Large | 10,000 | 123.98 | **68.97** | 258.03 | 1655.50 | acadrust |
| Huge | 100,000 | 845.86 | **419.98** | 1474.34 | 10234.18 | acadrust |

---

## Observations

- acadrust's parse speed advantage carries through to roundtrip performance.
- At huge scale, acadrust completes roundtrip in **420 ms** vs dxf-rs 846 ms, ACadSharp 1,474 ms, and ezdxf 10,234 ms.
- ezdxf's roundtrip is ~24× slower than acadrust at huge scale due to cumulative Python overhead on both read and write paths.
- Performance is consistent with v0.3.0 results, confirming no regressions.
