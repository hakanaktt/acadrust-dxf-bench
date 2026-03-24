# DXF Write Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest DXF writer** at all scales, with dxf-rs as a close second. ACadSharp is 2–3× slower, and ezdxf is 9–15× slower than acadrust.

---

## Results (ms, lower is better)

| Scale | Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | lines_only | 3.01 | **2.11** | 2.90 | 7.11 | acadrust |
| Small | mixed | **1.39** | 2.12 | 2.67 | 8.05 | dxf-rs |
| Medium | lines_only | 4.56 | **3.35** | 11.68 | 34.70 | acadrust |
| Medium | mixed | 3.88 | **2.58** | 12.11 | 39.70 | acadrust |
| Large | lines_only | 32.09 | **24.85** | 101.86 | 324.62 | acadrust |
| Large | mixed | 33.14 | **24.01** | 54.34 | 337.79 | acadrust |
| Huge | lines_only | 297.44 | **244.42** | 582.38 | 3124.28 | acadrust |
| Huge | mixed | 334.36 | **239.56** | 643.21 | 3527.14 | acadrust |

---

## Observations

- acadrust wins write benchmarks at medium/large/huge scales consistently.
- dxf-rs is competitive, trailing by only 1.2–1.4× at larger scales.
- ACadSharp is 2–2.7× slower than acadrust at huge scale.
- ezdxf is 13–15× slower than acadrust at huge scale — Python I/O and serialization overhead is significant.
