# DWG Roundtrip Performance Report

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

> **Note:** Only acadrust and ACadSharp support DWG format. dxf-rs and ezdxf do not support DWG.

---

## Summary

**acadrust is 2.9–3.3× faster than ACadSharp** for DWG roundtrip (read → write) across all scales.

---

## Results (ms, lower is better)

### dwg_mixed (read .dwg → write .dwg)

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **1.27** | 4.21 | acadrust |
| Medium | 1,000 | **4.67** | 13.53 | acadrust |
| Large | 10,000 | **45.05** | 144.47 | acadrust |
| Huge | 100,000 | **294.53** | 931.34 | acadrust |

---

## Observations

- acadrust wins every DWG roundtrip benchmark at every scale.
- The gap is consistent at 2.9–3.3× across all scales.
- acadrust achieves **295 ms** for 100,000 entities roundtrip — ACadSharp takes **931 ms**.
- DWG roundtrip is the most comprehensive test combining both parse and write performance.
