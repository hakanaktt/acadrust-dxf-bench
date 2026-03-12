# Binary DXF Roundtrip Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest at binary DXF roundtrip**, followed closely by dxf-rs. ACadSharp and ezdxf trail significantly due to managed-language overhead.

---

## Results (ms, lower is better)

### binary_mixed (read .dxb → write .dxb)

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.98 | **1.01** | 5.38 | 19.48 | dxf-rs |
| Medium | 1,000 | 3.55 | **2.57** | 30.07 | 78.67 | acadrust |
| Large | 10,000 | 32.07 | **20.55** | 140.67 | 761.33 | acadrust |
| Huge | 100,000 | 325.62 | **204.51** | 1572.55 | 9720.50 | acadrust |

---

## Observations

- At small scale dxf-rs narrowly beats acadrust, but acadrust pulls ahead at medium+ scales.
- acadrust is **1.6× faster than dxf-rs**, **7.7× faster than ACadSharp**, and **47× faster than ezdxf** at huge scale.
- ACadSharp binary roundtrip now reads and writes binary DXF (fixed from previous benchmark where it wrote ASCII output).
- ezdxf roundtrip requires upgrading the doc to R2018 (entity copy) since `ezdxf.readfile()` on .dxb returns docs without handles.
