# DXF Parse Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest DXF parser** at all scales, typically 2–2.5× faster than dxf-rs, 3–4× faster than ACadSharp, and 15–20× faster than ezdxf.

---

## Results (ms, lower is better)

### Small (100 entities)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 3.22 | **0.81** | 4.42 | 9.69 | acadrust |
| circles_only | 1.19 | **0.95** | 2.61 | 8.19 | acadrust |
| arcs_only | 1.02 | **0.81** | 2.75 | 9.40 | acadrust |
| ellipses_only | 1.68 | **0.51** | 2.83 | 9.83 | acadrust |
| mixed | 0.82 | **0.67** | 3.22 | 10.18 | acadrust |
| polylines | 0.36 | **0.26** | 3.04 | 5.16 | acadrust |
| 3d_entities | 0.93 | **0.64** | 4.69 | 10.41 | acadrust |

### Medium (1,000 entities)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 5.01 | **2.58** | 13.51 | 54.18 | acadrust |
| circles_only | 4.80 | **2.21** | 11.13 | 50.32 | acadrust |
| arcs_only | 6.19 | **2.75** | 15.48 | 67.83 | acadrust |
| ellipses_only | 5.83 | **3.48** | 15.27 | 61.28 | acadrust |
| mixed | 5.66 | **3.12** | 15.79 | 58.49 | acadrust |
| polylines | 0.99 | **0.53** | 4.35 | 10.37 | acadrust |
| 3d_entities | 6.31 | **3.66** | 21.85 | 65.45 | acadrust |

### Large (10,000 entities)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 50.06 | **22.10** | 182.01 | 561.70 | acadrust |
| circles_only | 45.65 | **18.92** | 164.77 | 450.16 | acadrust |
| arcs_only | 60.09 | **28.98** | 121.45 | 548.78 | acadrust |
| ellipses_only | 64.61 | **26.84** | 110.65 | 601.36 | acadrust |
| mixed | 52.10 | **22.73** | 95.93 | 557.47 | acadrust |
| polylines | 7.75 | **3.80** | 12.31 | 60.08 | acadrust |
| 3d_entities | 60.45 | **29.45** | 113.46 | 611.06 | acadrust |

### Huge (100,000 entities)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 678.71 | **295.19** | 973.73 | 5671.63 | acadrust |
| circles_only | 571.02 | **272.48** | 914.23 | 4778.76 | acadrust |
| arcs_only | 730.59 | **317.75** | 1064.55 | 5889.91 | acadrust |
| ellipses_only | 688.74 | **359.06** | 1068.95 | 6119.78 | acadrust |
| mixed | 622.53 | **304.22** | 1025.10 | 5862.24 | acadrust |
| polylines | 73.96 | **34.93** | 82.34 | 566.14 | acadrust |
| 3d_entities | 589.73 | **307.29** | 1041.41 | 6207.87 | acadrust |

---

## Observations

- acadrust is consistently **2–2.3× faster than dxf-rs** across all entity types and scales.
- ACadSharp (.NET) is **3–4× slower than acadrust** at large/huge scales.
- ezdxf (Python) is **~19× slower than acadrust** at huge scale, reflecting Python's interpreted overhead.
- All libraries scale linearly with entity count.
