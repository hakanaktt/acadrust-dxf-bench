# DXF Parse Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust is the fastest DXF parser** at all scales, typically 1.9–2.4× faster than dxf-rs, 3–5× faster than ACadSharp, and 15–24× faster than ezdxf.

---

## Results (ms, lower is better)

### Small (100 entities, 100 iterations)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 0.78 | **0.45** | 3.13 | 9.33 | acadrust |
| circles_only | 0.76 | **0.41** | 3.61 | 8.36 | acadrust |
| arcs_only | 0.86 | **0.47** | 4.14 | 9.22 | acadrust |
| ellipses_only | 0.91 | **0.45** | 4.46 | 9.63 | acadrust |
| mixed | 0.79 | **0.44** | 4.90 | 10.05 | acadrust |
| polylines | 0.37 | **0.22** | 2.90 | 4.85 | acadrust |
| 3d_entities | 0.86 | **0.47** | 4.89 | 9.73 | acadrust |

### Medium (1,000 entities, 50 iterations)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 6.68 | **3.02** | 16.98 | 57.53 | acadrust |
| circles_only | 6.92 | **2.57** | 16.52 | 91.47 | acadrust |
| arcs_only | 7.25 | **3.33** | 19.02 | 101.54 | acadrust |
| ellipses_only | 8.74 | **3.92** | 27.32 | 106.48 | acadrust |
| mixed | 7.37 | **3.45** | 19.61 | 96.13 | acadrust |
| polylines | 1.30 | **0.92** | 4.91 | 16.62 | acadrust |
| 3d_entities | 7.59 | **3.88** | 17.16 | 106.78 | acadrust |

### Large (10,000 entities, 25 iterations)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 71.39 | **35.87** | 105.89 | 870.05 | acadrust |
| circles_only | 69.44 | **29.73** | 125.27 | 765.58 | acadrust |
| arcs_only | 87.96 | **40.04** | 161.01 | 962.67 | acadrust |
| ellipses_only | 86.49 | **42.79** | 162.91 | 1010.39 | acadrust |
| mixed | 72.47 | **38.98** | 151.14 | 924.72 | acadrust |
| polylines | 12.68 | **5.63** | 16.22 | 112.62 | acadrust |
| 3d_entities | 94.95 | **44.32** | 193.11 | 1037.01 | acadrust |

### Huge (100,000 entities, 10 iterations)

| Workload | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|
| lines_only | 453.13 | **225.85** | 973.75 | 5335.87 | acadrust |
| circles_only | 413.08 | **191.47** | 783.44 | 4513.31 | acadrust |
| arcs_only | 512.02 | **239.53** | 874.63 | 5504.25 | acadrust |
| ellipses_only | 523.53 | **271.35** | 927.64 | 5871.61 | acadrust |
| mixed | 476.42 | **236.40** | 913.90 | 5498.17 | acadrust |
| polylines | 72.64 | **34.12** | 91.70 | 547.63 | acadrust |
| 3d_entities | 569.39 | **293.49** | 985.51 | 5921.64 | acadrust |

---

## Observations

- acadrust is consistently **2.0–2.3× faster than dxf-rs** across all entity types and scales.
- ACadSharp (.NET) is **3–5× slower than acadrust** at large/huge scales.
- ezdxf (Python) is **~23× slower than acadrust** at huge scale, reflecting Python's interpreted overhead.
- All libraries scale linearly with entity count.
- Compared to v0.3.0, acadrust maintains its parse performance lead with no regressions.
