# DXF Roundtrip Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest for DXF roundtrip** (read → write) at all scales, ~1.8× faster than dxf-rs, ~4× faster than ACadSharp, and ~25× faster than ezdxf at huge scale.

---

## Results (ms, lower is better)

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 3.74 | **1.94** | 6.43 | 17.19 | acadrust |
| Medium | 1,000 | 9.72 | **6.70** | 24.19 | 109.04 | acadrust |
| Large | 10,000 | 87.49 | **45.79** | 138.37 | 998.96 | acadrust |
| Huge | 100,000 | 873.02 | **482.14** | 1970.49 | 11964.53 | acadrust |

---

## Observations

- acadrust's parse speed advantage carries through to roundtrip performance.
- At huge scale, acadrust completes roundtrip in **482 ms** vs dxf-rs 873 ms, ACadSharp 1,970 ms, and ezdxf 11,965 ms.
- ezdxf's roundtrip is ~25× slower than acadrust at huge scale due to cumulative Python overhead on both read and write paths.
