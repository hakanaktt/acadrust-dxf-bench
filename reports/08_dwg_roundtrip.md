# DWG Roundtrip Performance Report

**Date:** March 13, 2026  
**Libraries:** acadrust v0.3.0, ACadSharp v3.4.9  
**Platform:** Windows, Release mode (`opt-level=3`, `lto=thin`)

> **Note:** Only acadrust and ACadSharp support DWG format. dxf-rs and ezdxf do not support DWG.

---

## Summary

**acadrust is 2.9–4.9× faster than ACadSharp** for DWG roundtrip (read → write) across all scales.

---

## Results (ms, lower is better)

### dwg_mixed (read .dwg → write .dwg)

| Scale | Entities | acadrust | ACadSharp | Fastest |
|---|---|---|---|---|
| Small | 100 | **1.63** | 4.77 | acadrust |
| Medium | 1,000 | **3.91** | 19.04 | acadrust |
| Large | 10,000 | **31.65** | 87.66 | acadrust |
| Huge | 100,000 | **312.07** | 1050.55 | acadrust |

---

## Observations

- acadrust wins every DWG roundtrip benchmark at every scale.
- The gap widens from 2.9× at small scale to 3.4× at huge scale.
- acadrust achieves **312 ms** for 100,000 entities roundtrip — ACadSharp takes over **1 second**.
- DWG roundtrip is the most comprehensive test combining both parse and write performance.
