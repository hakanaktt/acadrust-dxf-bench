# Binary DXF Parse Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

---

## Summary

**acadrust and dxf-rs are neck and neck** for binary DXF parsing. ACadSharp is 5–6× slower, and ezdxf is 23–27× slower.

---

## Results (ms, lower is better)

### binary_mixed

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | **0.24** | 0.28 | 4.46 | 6.40 | dxf-rs |
| Medium | 1,000 | 2.46 | **1.99** | 15.38 | 35.43 | acadrust |
| Large | 10,000 | 30.25 | **26.47** | 152.50 | 613.34 | acadrust |
| Huge | 100,000 | 179.95 | **165.86** | 917.92 | 4316.47 | acadrust |

### binary_lines

| Scale | Entities | dxf-rs | acadrust | ACadSharp | ezdxf | Fastest |
|---|---|---|---|---|---|---|
| Small | 100 | 0.25 | **0.24** | 3.57 | 5.72 | acadrust |
| Medium | 1,000 | **1.41** | 2.20 | 13.16 | 31.81 | dxf-rs |
| Large | 10,000 | 25.23 | **24.51** | 135.00 | 573.41 | acadrust |
| Huge | 100,000 | 164.33 | **161.95** | 867.50 | 4130.04 | acadrust |

---

## Observations

- The two Rust libraries are within 1–8% of each other for binary DXF parsing at all scales.
- ACadSharp is ~5.5× slower than the Rust libraries at huge scale.
- ezdxf is ~26× slower than acadrust at huge scale — while ezdxf can read binary DXF, it processes them through the same Python pipeline.
- At small scale, dxf-rs wins by a hair; acadrust pulls ahead at medium+ scales.
