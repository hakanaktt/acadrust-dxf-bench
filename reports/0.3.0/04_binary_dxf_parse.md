# Binary DXF Parse Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust and dxf-rs are neck and neck** for binary DXF parsing. ACadSharp is 5–6× slower, and ezdxf is 10–27× slower.

---

## Results (ms, lower is better)

### binary_mixed

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.64 | **0.58** | 3.61 | 6.12 | acadrust |
| Medium | 1,000 | 3.74 | **2.85** | 22.10 | 33.56 | acadrust |
| Large | 10,000 | 23.71 | **19.25** | 93.26 | 399.30 | acadrust |
| Huge | 100,000 | 189.14 | **184.06** | 1006.61 | 5010.33 | acadrust |

### binary_lines

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | **0.35** | 0.39 | 2.63 | 6.07 | dxf-rs |
| Medium | 1,000 | 2.36 | **2.18** | 17.78 | 29.92 | acadrust |
| Large | 10,000 | 22.75 | **18.36** | 95.65 | 360.67 | acadrust |
| Huge | 100,000 | 196.67 | **182.74** | 928.70 | 4871.84 | acadrust |

---

## Observations

- The two Rust libraries are within 3–8% of each other for binary DXF parsing at all scales.
- ACadSharp is ~5× slower than the Rust libraries at huge scale.
- ezdxf is ~27× slower than acadrust at huge scale — while ezdxf can read binary DXF, it processes them through the same Python pipeline.
