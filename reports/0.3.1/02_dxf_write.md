# DXF Write Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest DXF writer** at all scales, with dxf-rs as a close second. ACadSharp is 2–3× slower, and ezdxf is 9–15× slower than acadrust.

---

## Results (ms, lower is better)

| Scale | Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | lines_only | 0.77 | **0.56** | 2.87 | 6.72 | acadrust |
| Small | mixed | 0.72 | **0.54** | 2.76 | 7.08 | acadrust |
| Medium | lines_only | 4.57 | **3.37** | 19.65 | 61.54 | acadrust |
| Medium | mixed | 5.07 | **3.27** | 11.75 | 70.10 | acadrust |
| Large | lines_only | 47.85 | **37.84** | 98.22 | 547.23 | acadrust |
| Large | mixed | 51.42 | **34.45** | 87.19 | 579.54 | acadrust |
| Huge | lines_only | 289.52 | **233.90** | 523.70 | 2918.68 | acadrust |
| Huge | mixed | 303.54 | **222.57** | 520.29 | 3231.44 | acadrust |

---

## Observations

- acadrust wins write benchmarks at all scales consistently.
- dxf-rs is competitive, trailing by only 1.2–1.5× at larger scales.
- ACadSharp is 2.2–2.3× slower than acadrust at huge scale.
- ezdxf is 12–15× slower than acadrust at huge scale — Python I/O and serialization overhead is significant.
- Compared to v0.3.0, acadrust v0.3.1 shows consistent write performance.
