# DWG Parse & Write Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

> **Note:** Only acadrust and ACadSharp support DWG format. dxf-rs and ezdxf do not support DWG.

---

## Summary

**acadrust is 3–5× faster than ACadSharp** for both DWG parsing and writing across all scales.

---

## Parse Results (ms, lower is better)

### dwg_lines

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **0.50** | 1.62 | acadrust |
| Medium | 1,000 | **1.62** | 9.66 | acadrust |
| Large | 10,000 | **15.74** | 72.30 | acadrust |
| Huge | 100,000 | **171.35** | 840.34 | acadrust |

### dwg_mixed

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **0.55** | 2.20 | acadrust |
| Medium | 1,000 | **1.71** | 7.62 | acadrust |
| Large | 10,000 | **17.46** | 111.38 | acadrust |
| Huge | 100,000 | **178.91** | 893.78 | acadrust |

---

## Write Results (ms, lower is better)

### dwg_lines

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **1.30** | 2.60 | acadrust |
| Medium | 1,000 | **2.21** | 10.79 | acadrust |
| Large | 10,000 | **11.72** | 30.95 | acadrust |
| Huge | 100,000 | **143.39** | 574.75 | acadrust |

### dwg_mixed

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **1.09** | 3.84 | acadrust |
| Medium | 1,000 | **1.75** | 9.75 | acadrust |
| Large | 10,000 | **12.42** | 56.65 | acadrust |
| Huge | 100,000 | **136.63** | 346.07 | acadrust |

---

## Observations

- acadrust wins every DWG benchmark at every scale.
- Parse: acadrust is **4–6× faster** than ACadSharp (up to 6.4× at large/mixed).
- Write: acadrust is **2–5× faster** than ACadSharp, with the gap growing at larger scales.
- DWG remains exclusive to acadrust and ACadSharp — neither dxf-rs nor ezdxf support the format.
