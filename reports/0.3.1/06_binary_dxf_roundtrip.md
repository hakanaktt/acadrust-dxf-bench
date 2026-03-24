# Binary DXF Roundtrip Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest at binary DXF roundtrip**, followed by dxf-rs. ACadSharp and ezdxf trail significantly due to managed-language overhead.

---

## Results (ms, lower is better)

### binary_mixed (read .dxb → write .dxb)

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.82 | **0.64** | 5.37 | 14.98 | acadrust |
| Medium | 1,000 | 4.86 | **3.44** | 21.69 | 113.62 | acadrust |
| Large | 10,000 | 47.34 | **29.93** | 218.66 | 1288.99 | acadrust |
| Huge | 100,000 | 338.09 | **198.91** | 1230.68 | 8339.10 | acadrust |

---

## Observations

- acadrust wins at every scale for binary DXF roundtrip.
- acadrust is **1.7× faster than dxf-rs**, **6.2× faster than ACadSharp**, and **42× faster than ezdxf** at huge scale.
- ACadSharp binary roundtrip reads and writes binary DXF correctly.
- ezdxf roundtrip requires upgrading the doc to R2018 (entity copy) since `ezdxf.readfile()` on .dxb returns docs without handles.
