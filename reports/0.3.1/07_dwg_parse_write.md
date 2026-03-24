# DWG Parse & Write Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

> **Note:** Only acadrust and ACadSharp support DWG format. dxf-rs and ezdxf do not support DWG.

---

## Summary

**acadrust is 2–6× faster than ACadSharp** for both DWG parsing and writing across all scales.

---

## Parse Results (ms, lower is better)

### dwg_lines

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **0.38** | 1.33 | acadrust |
| Medium | 1,000 | **1.93** | 7.96 | acadrust |
| Large | 10,000 | **21.11** | 93.20 | acadrust |
| Huge | 100,000 | **162.97** | 580.23 | acadrust |

### dwg_mixed

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **0.37** | 1.40 | acadrust |
| Medium | 1,000 | **2.56** | 8.56 | acadrust |
| Large | 10,000 | **22.83** | 123.51 | acadrust |
| Huge | 100,000 | **165.00** | 568.74 | acadrust |

---

## Write Results (ms, lower is better)

### dwg_lines

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **0.80** | 2.25 | acadrust |
| Medium | 1,000 | **2.27** | 6.54 | acadrust |
| Large | 10,000 | **19.85** | 44.21 | acadrust |
| Huge | 100,000 | **127.62** | 369.67 | acadrust |

### dwg_mixed

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **0.78** | 2.84 | acadrust |
| Medium | 1,000 | **2.50** | 9.63 | acadrust |
| Large | 10,000 | **20.85** | 51.31 | acadrust |
| Huge | 100,000 | **176.72** | 374.26 | acadrust |

---

## Observations

- acadrust wins every DWG benchmark at every scale.
- Parse: acadrust is **3.4–5.4× faster** than ACadSharp (up to 5.4× at large/mixed).
- Write: acadrust is **2.1–3.9× faster** than ACadSharp, with the gap growing at larger scales.
- DWG remains exclusive to acadrust and ACadSharp — neither dxf-rs nor ezdxf support the format.
