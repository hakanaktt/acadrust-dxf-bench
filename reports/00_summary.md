# Overall Benchmark Summary

**Date:** March 12, 2026  
**Libraries:** acadrust v0.3.0, dxf-rs v0.6, ACadSharp v3.4.9  
**Platform:** Windows, Release mode  
**Scales tested:** Small (100), Medium (1,000), Large (10,000), Huge (100,000 entities)

---

## Category Winners at a Glance

| Category | Fastest Library | Margin | Notes |
|---|---|---|---|
| **DXF Parse** | acadrust | 1.4–3.7× vs dxf-rs, 2–8× vs ACadSharp | Clear leader at all scales |
| **DXF Write** | acadrust / dxf-rs | Mixed — dxf-rs wins at medium scale | Close race, scale-dependent |
| **DXF Roundtrip** | acadrust | 1.8–2.9× vs dxf-rs, 3–8× vs ACadSharp | Parse advantage dominates |
| **Binary DXF Parse** | acadrust ≈ dxf-rs | Neck and neck; ACadSharp 4–8× slower | Rust libraries both strong |
| **Binary DXF Write** | acadrust | 1.7–9× vs dxf-rs | ACadSharp has no binary DXF write |
| **Binary DXF Roundtrip** | acadrust | ~2× vs dxf-rs, 5–7× vs ACadSharp | ACadSharp read-only for binary |
| **DWG Parse** | acadrust | 2–4.5× vs ACadSharp | dxf-rs has no DWG support |
| **DWG Write** | acadrust | 1.5–6× vs ACadSharp | dxf-rs has no DWG support |
| **DWG Roundtrip** | acadrust | ~2.7× vs ACadSharp | dxf-rs has no DWG support |

---

## Format Support Matrix

| Feature | dxf-rs | acadrust | ACadSharp |
|---|---|---|---|
| DXF Read | ✅ | ✅ | ✅ |
| DXF Write | ✅ | ✅ | ✅ |
| Binary DXF Read | ✅ | ✅ | ✅ |
| Binary DXF Write | ✅ | ✅ | ❌ |
| DWG Read | ❌ | ✅ | ✅ |
| DWG Write | ❌ | ✅ | ✅ |

---

## Key Takeaways

1. **acadrust is the overall fastest library**, winning or tying in every benchmark category.
2. **dxf-rs** is competitive for binary DXF parsing (matching acadrust) and occasionally wins DXF writing at mid-range scales, but falls behind on parsing.
3. **ACadSharp** offers the broadest .NET ecosystem compatibility but is consistently slower — typically 3–8× behind acadrust for parsing and 2–6× for writing.
4. The performance gap **widens at larger scales**, suggesting acadrust's native Rust implementation scales more efficiently with data size.
5. **DWG benchmarks** are exclusively acadrust vs ACadSharp (dxf-rs has no DWG support), with acadrust leading by 2–4.5× across the board.

---

## Detailed Reports

1. [DXF Parse](01_dxf_parse.md)
2. [DXF Write](02_dxf_write.md)
3. [DXF Roundtrip](03_dxf_roundtrip.md)
4. [Binary DXF Parse](04_binary_dxf_parse.md)
5. [Binary DXF Write](05_binary_dxf_write.md)
6. [Binary DXF Roundtrip](06_binary_dxf_roundtrip.md)
7. [DWG Parse & Write](07_dwg_parse_write.md)
8. [DWG Roundtrip](08_dwg_roundtrip.md)
