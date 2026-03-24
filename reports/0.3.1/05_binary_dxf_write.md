# Binary DXF Write Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust dominates binary DXF writing**, 4–5× faster than dxf-rs, 8–13× faster than ACadSharp, and 66–112× faster than ezdxf at huge scale.

---

## Results (ms, lower is better)

### binary_lines

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.45 | **0.27** | 2.12 | 6.47 | acadrust |
| Medium | 1,000 | 1.49 | **0.62** | 7.43 | 38.22 | acadrust |
| Large | 10,000 | 16.39 | **3.98** | 49.37 | 512.57 | acadrust |
| Huge | 100,000 | 107.13 | **28.01** | 351.44 | 2777.99 | acadrust |

### binary_mixed

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.37 | **0.26** | 1.90 | 6.93 | acadrust |
| Medium | 1,000 | 1.84 | **0.80** | 6.65 | 53.03 | acadrust |
| Large | 10,000 | 17.97 | **3.39** | 56.10 | 553.16 | acadrust |
| Huge | 100,000 | 114.79 | **27.00** | 344.76 | 3028.39 | acadrust |

---

## Observations

- acadrust's binary write is exceptionally fast — at huge scale it writes binary DXF **3.8× faster than dxf-rs**, **12.5× faster than ACadSharp**, and **99–112× faster than ezdxf**.
- ACadSharp supports binary DXF writing (using `DxfWriter` with `isBinary=true`).
- ezdxf writes binary DXF using `doc.saveas(path, fmt='bin')` but Python overhead makes it the slowest option.
- Performance consistent with v0.3.0.
