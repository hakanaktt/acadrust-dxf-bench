# Binary DXF Write Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust dominates binary DXF writing**, 3–5× faster than dxf-rs, 8–10× faster than ACadSharp, and 80–85× faster than ezdxf at huge scale.

---

## Results (ms, lower is better)

### binary_lines

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.84 | **0.47** | 2.31 | 7.02 | acadrust |
| Medium | 1,000 | 1.54 | **0.52** | 9.25 | 33.43 | acadrust |
| Large | 10,000 | 17.48 | **2.36** | 35.54 | 290.34 | acadrust |
| Huge | 100,000 | 114.47 | **28.91** | 329.05 | 3077.72 | acadrust |

### binary_mixed

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.45 | **0.25** | 2.11 | 11.52 | acadrust |
| Medium | 1,000 | 1.34 | **0.44** | 9.60 | 35.69 | acadrust |
| Large | 10,000 | 11.76 | **2.32** | 31.43 | 312.02 | acadrust |
| Huge | 100,000 | 139.70 | **38.67** | 405.51 | 3266.13 | acadrust |

---

## Observations

- acadrust's binary write is exceptionally fast — at huge scale it writes binary DXF **4× faster than dxf-rs**, **10× faster than ACadSharp**, and **84× faster than ezdxf**.
- ACadSharp now supports binary DXF writing (using `DxfWriter` with `isBinary=true`), filling a gap from previous benchmarks.
- ezdxf writes binary DXF using `doc.saveas(path, fmt='bin')` but Python overhead makes it the slowest option.
